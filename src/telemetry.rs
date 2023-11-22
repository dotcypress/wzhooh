use usb_device::class_prelude::*;
use crate::counter::TrackStats;

pub const REPORT_SIZE: usize = 8;

pub enum AppRequest {
    SendCounterState,
    ResetCounter,
}

pub struct RaceTelemetryClass<'a, B: UsbBus> {
    iface: InterfaceNumber,
    ep: EndpointIn<'a, B>,
    app_req: Option<AppRequest>,
    reports: heapless::Deque<[u8; REPORT_SIZE], 32>,
}

impl<B: UsbBus> RaceTelemetryClass<'_, B> {
    pub fn new(alloc: &UsbBusAllocator<B>) -> RaceTelemetryClass<'_, B> {
        RaceTelemetryClass {
            iface: alloc.interface(),
            ep: alloc.interrupt(REPORT_SIZE as _, 10),
            app_req: None,
            reports: heapless::Deque::new(),
        }
    }

    pub fn send_reset(&mut self) {
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
        report[0] = 0xfe;
        report[1] = track;
        report[2..4].copy_from_slice(&laps.to_be_bytes());
        report[4..6].copy_from_slice(&last.to_be_bytes());
        report[6..8].copy_from_slice(&best.to_be_bytes());
        self.reports.push_front(report).ok();
    }

    pub fn app_req(&mut self) -> Option<AppRequest> {
        self.app_req.take()
    }

    pub fn send_report(&mut self) {
        let report = self.reports.pop_back().unwrap_or([0; REPORT_SIZE]);
        self.ep.write(&report).ok();
    }
}

impl<B: UsbBus> UsbClass<B> for RaceTelemetryClass<'_, B> {
    fn get_configuration_descriptors(
        &self,
        writer: &mut DescriptorWriter,
    ) -> usb_device::Result<()> {
        writer.interface(self.iface, 0xff, 0x00, 0x00)?;
        writer.endpoint(&self.ep)?;
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
                self.reports.clear();
                self.app_req = Some(AppRequest::ResetCounter);
                xfer.accept().ok()
            }
            0x02 => {
                self.app_req = Some(AppRequest::SendCounterState);
                xfer.accept().ok()
            }
            _ => xfer.reject().ok(),
        };
    }
}
