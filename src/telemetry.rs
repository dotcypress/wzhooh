use usb_device::class_prelude::*;

use crate::counter::TrackStats;

pub const REPORT_SIZE: usize = 8;

pub enum AppRequest {
    ResetCounter,
}

pub struct RaceTelemetryClass<'a, B: UsbBus> {
    iface: InterfaceNumber,
    ep_interrupt_in: EndpointIn<'a, B>,
    reports: heapless::Deque<[u8; REPORT_SIZE], 16>,
    reset_req: bool,
}

impl<B: UsbBus> RaceTelemetryClass<'_, B> {
    pub fn new(alloc: &UsbBusAllocator<B>) -> RaceTelemetryClass<'_, B> {
        RaceTelemetryClass {
            iface: alloc.interface(),
            ep_interrupt_in: alloc.interrupt(REPORT_SIZE as _, 10),
            reports: heapless::Deque::new(),
            reset_req: false,
        }
    }

    pub fn push_reset(&mut self) {
        self.reports.push_front([0xff; REPORT_SIZE]).ok();
    }

    pub fn push_track_stats(&mut self, stats: TrackStats) {
        let track = stats.track() as u8;
        let laps = stats.laps() as u16;
        let last = stats
            .last()
            .map(|dur| dur.to_millis() as u16)
            .unwrap_or_default();
        let best = stats
            .best()
            .map(|dur| dur.to_millis() as u16)
            .unwrap_or_default();
        let mut report = [0; REPORT_SIZE];
        report[0] = 0x01;
        report[1] = track;
        report[2..4].copy_from_slice(&laps.to_be_bytes());
        report[4..6].copy_from_slice(&last.to_be_bytes());
        report[6..8].copy_from_slice(&best.to_be_bytes());
        self.reports.push_front(report).ok();
    }

    pub fn poll(&mut self) -> Option<AppRequest> {
        let report = self.reports.pop_back().unwrap_or([0; REPORT_SIZE]);
        self.ep_interrupt_in.write(&report).ok();
        if self.reset_req {
            self.reset_req = false;
            Some(AppRequest::ResetCounter)
        } else {
            None
        }
    }
}

impl<B: UsbBus> UsbClass<B> for RaceTelemetryClass<'_, B> {
    fn get_configuration_descriptors(
        &self,
        writer: &mut DescriptorWriter,
    ) -> usb_device::Result<()> {
        writer.interface(self.iface, 0xff, 0x00, 0x00)?;
        writer.endpoint(&self.ep_interrupt_in)?;
        Ok(())
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = xfer.request();
        if !(req.request_type == control::RequestType::Vendor
            && req.recipient == control::Recipient::Device)
        {
            return;
        }

        match req.request {
            0x01 => {
                self.reset_req = true;
                xfer.accept().unwrap();
            }
            _ => xfer.reject().unwrap(),
        }
    }
}
