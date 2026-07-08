// SPDX-License-Identifier: MPL-2.0

mod table;

use core::fmt::Debug;

use spin::Once;
use table::ExtendedInterruptMode;
pub(super) use table::IntRemappingTable;

use crate::{
    arch::{
        iommu::registers::{ExtendedCapabilityFlags, IOMMU_REGS},
        kernel::apic::{self, ApicId},
    },
    info, warn,
};

pub struct IrtEntryHandle {
    index: u16,
    table: &'static IntRemappingTable,
}

impl IrtEntryHandle {
    pub fn index(&self) -> u16 {
        self.index
    }

    pub fn enable(&self, vector: u32) -> bool {
        if !self
            .table
            .set_enabled_entry(self.index, vector, apic::current_id())
        {
            warn!("failed to encode interrupt-remapping destination");
            return false;
        }

        IOMMU_REGS
            .get()
            .unwrap()
            .lock()
            .invalidate_interrupt_cache();
        true
    }
}

impl Debug for IrtEntryHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IrtEntryHandle")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}

pub fn has_interrupt_remapping() -> bool {
    REMAPPING_TABLE.get().is_some()
}

pub fn alloc_irt_entry() -> Option<IrtEntryHandle> {
    let page_table = REMAPPING_TABLE.get()?;
    page_table.alloc()
}

pub(super) fn init() {
    let mut iommu_regs = IOMMU_REGS.get().unwrap().lock();

    // Check if interrupt remapping is supported
    let extend_cap = iommu_regs.read_extended_capability();
    if !extend_cap.flags().contains(ExtendedCapabilityFlags::IR) {
        warn!("Interrupt remapping not supported");
        return;
    }
    let extended_interrupt_mode = if extend_cap.flags().contains(ExtendedCapabilityFlags::EIM)
        && matches!(apic::current_id(), ApicId::X2Apic(_))
    {
        ExtendedInterruptMode::X2Apic
    } else {
        ExtendedInterruptMode::XApic
    };

    // Create interrupt remapping table
    REMAPPING_TABLE.call_once(|| IntRemappingTable::new(extended_interrupt_mode));
    iommu_regs.enable_interrupt_remapping(REMAPPING_TABLE.get().unwrap());

    info!("Interrupt remapping enabled");
}

static REMAPPING_TABLE: Once<IntRemappingTable> = Once::new();
