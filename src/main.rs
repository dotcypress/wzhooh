#![no_std]
#![no_main]

extern crate panic_probe;
extern crate rp2040_hal as hal;
extern crate rtic;

use defmt_rtt as _;

use cortex_m::singleton;
use hal::gpio::*;
use hal::pac;
use hal::usb::UsbBus;
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

pub const XTAL_FREQ_HZ: u32 = 12_000_000_u32;

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[rtic::app(device = pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

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
        let _pins = Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );

        let usb_regs = ctx.device.USBCTRL_REGS;
        let usb_dpram = ctx.device.USBCTRL_DPRAM;
        let usb_bus = UsbBus::new(usb_regs, usb_dpram, clocks.usb_clock, true, &mut resets);
        let usb_bus: &'static UsbBusAllocator<UsbBus> =
            singleton!(: UsbBusAllocator<UsbBus> = UsbBusAllocator::new(usb_bus)).unwrap();

        let _serial = SerialPort::new(usb_bus);
        let _usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Ferris & Co")
            .product("vitaly.codes/slot-car-lap-timer")
            .serial_number("_sclt_")
            .device_class(2)
            .build();

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ);
        };

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task(binds = USBCTRL_IRQ, shared = [])]
    fn usb_irq(_: usb_irq::Context) {}
}
