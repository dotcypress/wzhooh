#![no_std]
#![no_main]

extern crate panic_probe;
extern crate rp2040_hal as hal;
extern crate rtic;

use defmt_rtt as _;

mod counter;
mod display;
mod io;

use cortex_m::singleton;
use counter::*;
use display::*;
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::*;
use hal::pac;
use hal::pwm;
use hal::timer::{monotonic::Monotonic, *};
use hal::usb::UsbBus;
use io::*;
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

pub const XTAL_FREQ_HZ: u32 = 12_000_000_u32;

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[rtic::app(device = pac, peripherals = true, dispatchers = [SW0_IRQ])]
mod app {
    use super::*;

    #[monotonic(binds = TIMER_IRQ_0, default = true)]
    type Oracle = Monotonic<Alarm0>;

    #[local]
    struct Local {
        buttons: Buttons,
        sensors: Sensors,
        ui_timer: pwm::Slice<pwm::Pwm0, pwm::FreeRunning>,
    }

    #[shared]
    struct Shared {
        display: Display,
        counter: LapCounter,
        serial: (
            SerialPort<'static, hal::usb::UsbBus>,
            UsbDevice<'static, hal::usb::UsbBus>,
        ),
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut resets = ctx.device.RESETS;
        let mut watchdog = hal::Watchdog::new(ctx.device.WATCHDOG);
        let clocks = hal::clocks::init_clocks_and_plls(
            XTAL_FREQ_HZ,
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let sio = hal::Sio::new(ctx.device.SIO);
        let pins = Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );

        let counter = LapCounter::default();

        let sensors = Sensors::new([
            pins.gpio28.into_floating_input().into_dyn_pin(),
            pins.gpio27.into_floating_input().into_dyn_pin(),
            pins.gpio26.into_floating_input().into_dyn_pin(),
        ]);

        let buttons = Buttons::new([
            pins.gpio7.into_pull_up_input().into_dyn_pin(),
            pins.gpio6.into_pull_up_input().into_dyn_pin(),
            pins.gpio5.into_pull_up_input().into_dyn_pin(),
        ]);

        let display = Display::new(
            [
                pins.gpio20.into_push_pull_output().into_dyn_pin(),
                pins.gpio21.into_push_pull_output().into_dyn_pin(),
                pins.gpio18.into_push_pull_output().into_dyn_pin(),
                pins.gpio19.into_push_pull_output().into_dyn_pin(),
                pins.gpio16.into_push_pull_output().into_dyn_pin(),
                pins.gpio17.into_push_pull_output().into_dyn_pin(),
            ],
            [
                pins.gpio8.into_push_pull_output().into_dyn_pin(),
                pins.gpio9.into_push_pull_output().into_dyn_pin(),
                pins.gpio10.into_push_pull_output().into_dyn_pin(),
                pins.gpio11.into_push_pull_output().into_dyn_pin(),
                pins.gpio12.into_push_pull_output().into_dyn_pin(),
                pins.gpio13.into_push_pull_output().into_dyn_pin(),
                pins.gpio14.into_push_pull_output().into_dyn_pin(),
                pins.gpio15.into_push_pull_output().into_dyn_pin(),
            ],
        );

        let pwm_slices = hal::pwm::Slices::new(ctx.device.PWM, &mut resets);
        let mut ui_timer = pwm_slices.pwm0;
        ui_timer.enable_interrupt();
        ui_timer.enable();

        let mut timer = hal::Timer::new(ctx.device.TIMER, &mut resets, &clocks);
        let alarm = timer.alarm_0().unwrap();
        let mono = Monotonic::new(timer, alarm);

        let usb_regs = ctx.device.USBCTRL_REGS;
        let usb_dpram = ctx.device.USBCTRL_DPRAM;
        let usb_bus = UsbBus::new(usb_regs, usb_dpram, clocks.usb_clock, true, &mut resets);
        let usb_bus: &'static UsbBusAllocator<UsbBus> =
            singleton!(: UsbBusAllocator<UsbBus> = UsbBusAllocator::new(usb_bus)).unwrap();

        let serial = (
            SerialPort::new(usb_bus),
            UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
                .manufacturer("Ferris & Co")
                .product("vitaly.codes/wzhooh")
                .serial_number("_wzhooh_")
                .device_class(2)
                .build(),
        );

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
            pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ);
        };

        (
            Shared {
                counter,
                display,
                serial,
            },
            Local {
                buttons,
                sensors,
                ui_timer,
            },
            init::Monotonics(mono),
        )
    }

    #[task(binds = IO_IRQ_BANK0, priority = 2, local = [buttons, sensors])]
    fn io_irq(ctx: io_irq::Context) {
        for track in [Track::A, Track::B, Track::C] {
            if ctx.local.sensors.is_car_detected(track) {
                io_event::spawn(IoEvent::CarDetected(track, monotonics::now())).ok();
            }
        }
        for button in [Button::A, Button::B, Button::C] {
            if ctx.local.buttons.is_pressed(button) {
                io_event::spawn(IoEvent::ButtonPressed(button)).ok();
            }
        }
    }

    #[task(capacity = 16, shared = [counter, display, serial])]
    fn io_event(ctx: io_event::Context, ev: IoEvent) {
        let io_event::SharedResources {
            mut counter,
            mut display,
            mut serial,
        } = ctx.shared;

        match ev {
            IoEvent::ButtonPressed(_) => {
                counter.lock(|counter| counter.reset());
                display.lock(|display| display.reset());
            }
            IoEvent::CarDetected(track, ts) => {
                let laps = counter.lock(|counter| counter.record_lap(track, ts));
                display.lock(|display| display.set_track_laps(track, laps));
                serial.lock(|(serial, _)| {
                    use core::fmt::Write;
                    let mut buf = heapless::String::<64>::default();
                    write!(&mut buf, "Track #{}: {}\r\n", track as u8, laps).ok();
                    serial.write(buf.as_bytes()).ok();
                });
            }
        }
    }

    #[task(binds = PWM_IRQ_WRAP, local = [ui_timer], shared = [display])]
    fn update_ui(mut ctx: update_ui::Context) {
        ctx.shared.display.lock(|display| display.animate());
        ctx.local.ui_timer.clear_interrupt();
    }

    #[task(binds = USBCTRL_IRQ, shared = [serial])]
    fn usb_irq(mut ctx: usb_irq::Context) {
        ctx.shared.serial.lock(|(serial, usb)| {
            if usb.poll(&mut [serial]) {
                let mut scratch = [0; 64];
                match serial.read(&mut scratch) {
                    Ok(n) if n > 0 => {}
                    _ => {}
                }
            }
        });
    }
}
