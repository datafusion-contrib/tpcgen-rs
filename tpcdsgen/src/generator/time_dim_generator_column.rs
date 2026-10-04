use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Generator columns for TIME_DIM table (TimeDimGeneratorColumn)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeDimGeneratorColumn {
    TTimeSk,
    TTimeId,
    TTime,
    THour,
    TMinute,
    TSecond,
    TAmPm,
    TShift,
    TSubShift,
    TMealTime,
    TNulls,
}

impl TimeDimGeneratorColumn {
    pub fn values() -> &'static [TimeDimGeneratorColumn] {
        use TimeDimGeneratorColumn::*;
        static VALUES: &[TimeDimGeneratorColumn] = &[
            TTimeSk, TTimeId, TTime, THour, TMinute, TSecond, TAmPm, TShift, TSubShift, TMealTime,
            TNulls,
        ];
        VALUES
    }

    fn get_column_info(&self) -> (i32, u32) {
        use TimeDimGeneratorColumn::*;
        match self {
            TTimeSk => (340, 1),
            TTimeId => (341, 1),
            TTime => (342, 1),
            THour => (343, 1),
            TMinute => (344, 1),
            TSecond => (345, 1),
            TAmPm => (346, 1),
            TShift => (347, 1),
            TSubShift => (348, 1),
            TMealTime => (349, 1),
            TNulls => (350, 1),
        }
    }
}

impl GeneratorColumn for TimeDimGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::TimeDim
    }

    fn get_global_column_number(&self) -> i32 {
        self.get_column_info().0
    }

    fn get_seeds_per_row(&self) -> u32 {
        self.get_column_info().1
    }
}
