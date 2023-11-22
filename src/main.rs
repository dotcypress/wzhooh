#![no_std]
#![no_main]

extern crate panic_probe;
extern crate rp2040_hal as hal;
extern crate rtic;

use defmt_rtt as _;

mod counter;
mod display;
mod io;
mod telemetry;

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
use telemetry::*;
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_webusb::{url_scheme, WebUsb};

pub const XTAL_FREQ_HZ: u32 = 12_000_000_u32;

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[rtic::app(device = pac, peripherals = true, dispatchers = [SW0_IRQ, SW1_IRQ])]
mod app {
    use super::*;

    #[monotonic(binds = TIMER_IRQ_0, default = true)]
    type Oracle = Monotonic<Alarm0>;

    #[local]
    struct Local {
        buttons: Buttons,
        sensors: Sensors,
        ui_timer: pwm::Slice<pwm::Pwm0, pwm::FreeRunning>,
        usb_dev: UsbDevice<'static, hal::usb::UsbBus>,
        wusb: WebUsb<hal::usb::UsbBus>,
    }

    #[shared]
    struct Shared {
        display: Display,
        counter: LapCounter,
        telemetry: RaceTelemetryClass<'static, hal::usb::UsbBus>,
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
        .expect("Clocks init failed");

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
        let alarm = timer.alarm_0().expect("Alarm0 init failed");
        let mono = Monotonic::new(timer, alarm);

        let usb_regs = ctx.device.USBCTRL_REGS;
        let usb_dpram = ctx.device.USBCTRL_DPRAM;
        let usb_bus = UsbBus::new(usb_regs, usb_dpram, clocks.usb_clock, true, &mut resets);
        let usb_bus: &'static UsbBusAllocator<UsbBus> =
            singleton!(: UsbBusAllocator<UsbBus> = UsbBusAllocator::new(usb_bus))
                .expect("USB init failed");

        let wusb = WebUsb::new(usb_bus, url_scheme::HTTPS, "wzhooh.vercel.app");
        let telemetry = RaceTelemetryClass::new(usb_bus);

        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x1209, 0xb421))
            .manufacturer("vitaly.codes")
            .product("Wzhooh")
            .serial_number("_wzhooh_")
            .build();

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
            pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ);
        };

        (
            Shared {
                counter,
                display,
                telemetry,
            },
            Local {
                buttons,
                sensors,
                wusb,
                ui_timer,
                usb_dev,
            },
            init::Monotonics(mono),
        )
    }

    #[task(binds = IO_IRQ_BANK0, priority = 3, local = [buttons, sensors])]
    fn io_irq(ctx: io_irq::Context) {
        let now = monotonics::now();
        for track in 0..TRACKS {
            if ctx.local.sensors.is_car_detected(track) {
                io_event::spawn(IoEvent::CarDetected(track, now)).ok();
            }
        }
        for button in [Button::A, Button::B, Button::C] {
            if ctx.local.buttons.is_pressed(button) {
                io_event::spawn(IoEvent::ButtonPressed(button)).ok();
            }
        }
    }

    #[task(binds = USBCTRL_IRQ, priority = 2, local = [wusb, usb_dev], shared = [telemetry])]
    fn usb_irq(ctx: usb_irq::Context) {
        let mut telemetry = ctx.shared.telemetry;
        telemetry.lock(|telemetry| {
            if ctx.local.usb_dev.poll(&mut [ctx.local.wusb, telemetry]) {
                telemetry.send_report();
            }
            telemetry.app_req().map(app_req::spawn);
        });
    }

    #[task(binds = PWM_IRQ_WRAP, local = [ui_timer], shared = [display])]
    fn update_ui(mut ctx: update_ui::Context) {
        ctx.shared.display.lock(|display| display.animate());
        ctx.local.ui_timer.clear_interrupt();
    }

    #[task(capacity = 8, shared = [counter, display, telemetry])]
    fn app_req(ctx: app_req::Context, req: AppRequest) {
        let app_req::SharedResources {
            mut counter,
            mut display,
            mut telemetry,
        } = ctx.shared;

        match req {
            AppRequest::ResetCounter => {
                counter.lock(|counter| counter.reset());
                display.lock(|display| display.reset());
                telemetry.lock(|telemetry| telemetry.send_reset());
            }
            AppRequest::SendCounterState => {
                telemetry.lock(|telemetry| telemetry.send_reset());
                for track in 0..TRACKS {
                    if let Some(stats) = counter.lock(|counter| counter.stats(track)) {
                        telemetry.lock(|telemetry| telemetry.push_track_stats(stats))
                    }
                }
            }
        }
    }

    #[task(capacity = 16, shared = [counter, display, telemetry])]
    fn io_event(ctx: io_event::Context, ev: IoEvent) {
        let io_event::SharedResources {
            mut counter,
            mut display,
            mut telemetry,
        } = ctx.shared;

        match ev {
            IoEvent::CarDetected(track, ts) => {
                if let Some(stats) = counter.lock(|counter| counter.record_lap(track, ts)) {
                    display.lock(|display| display.set_track_laps(track, stats.laps()));
                    telemetry.lock(|telemetry| telemetry.push_track_stats(stats));
                }
            }
            IoEvent::ButtonPressed(_) => {
                app_req::spawn(AppRequest::ResetCounter).ok();
            }
        }
    }
}
