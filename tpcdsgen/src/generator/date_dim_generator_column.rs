use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Generator columns for DATE_DIM table (DateDimGeneratorColumn)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateDimGeneratorColumn {
    DDateSk,
    DDateId,
    DDate,
    DMonthSeq,
    DWeekSeq,
    DQuarterSeq,
    DYear,
    DDow,
    DMoy,
    DDom,
    DQoy,
    DFyYear,
    DFyQuarterSeq,
    DFyWeekSeq,
    DDayName,
    DQuarterName,
    DHoliday,
    DWeekend,
    DFollowingHoliday,
    DFirstDom,
    DLastDom,
    DSameDayLy,
    DSameDayLq,
    DCurrentDay,
    DCurrentWeek,
    DCurrentMonth,
    DCurrentQuarter,
    DCurrentYear,
    DNulls,
}

impl DateDimGeneratorColumn {
    pub fn values() -> &'static [DateDimGeneratorColumn] {
        use DateDimGeneratorColumn::*;
        static VALUES: &[DateDimGeneratorColumn] = &[
            DDateSk,
            DDateId,
            DDate,
            DMonthSeq,
            DWeekSeq,
            DQuarterSeq,
            DYear,
            DDow,
            DMoy,
            DDom,
            DQoy,
            DFyYear,
            DFyQuarterSeq,
            DFyWeekSeq,
            DDayName,
            DQuarterName,
            DHoliday,
            DWeekend,
            DFollowingHoliday,
            DFirstDom,
            DLastDom,
            DSameDayLy,
            DSameDayLq,
            DCurrentDay,
            DCurrentWeek,
            DCurrentMonth,
            DCurrentQuarter,
            DCurrentYear,
            DNulls,
        ];
        VALUES
    }

    fn get_column_info(&self) -> (i32, u32) {
        use DateDimGeneratorColumn::*;
        match self {
            DDateSk => (159, 0),
            DDateId => (160, 0),
            DDate => (161, 0),
            DMonthSeq => (162, 0),
            DWeekSeq => (163, 0),
            DQuarterSeq => (164, 0),
            DYear => (165, 0),
            DDow => (166, 0),
            DMoy => (167, 0),
            DDom => (168, 0),
            DQoy => (169, 0),
            DFyYear => (170, 0),
            DFyQuarterSeq => (171, 0),
            DFyWeekSeq => (172, 0),
            DDayName => (173, 0),
            DQuarterName => (174, 0),
            DHoliday => (175, 0),
            DWeekend => (176, 0),
            DFollowingHoliday => (177, 0),
            DFirstDom => (178, 0),
            DLastDom => (179, 0),
            DSameDayLy => (180, 0),
            DSameDayLq => (181, 0),
            DCurrentDay => (182, 0),
            DCurrentWeek => (183, 0),
            DCurrentMonth => (184, 0),
            DCurrentQuarter => (185, 0),
            DCurrentYear => (186, 0),
            DNulls => (187, 2),
        }
    }
}

impl GeneratorColumn for DateDimGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::DateDim
    }

    fn get_global_column_number(&self) -> i32 {
        self.get_column_info().0
    }

    fn get_seeds_per_row(&self) -> u32 {
        self.get_column_info().1
    }
}
