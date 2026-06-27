use crate::parser::running_queries;

pub enum RunningQueriesIter<'a> {
    PGSQL(PGSQLIterator<'a>),
    MSSQL(MSSQLIterator<'a>),
}

impl<'a> RunningQueriesIter<'a> {
    pub fn new(strategy: running_queries::Strategy, body: &'a [u8]) -> Self {
        match strategy {
            running_queries::Strategy::PGSQL => RunningQueriesIter::PGSQL(PGSQLIterator(body)),
            running_queries::Strategy::MSSQL => RunningQueriesIter::MSSQL(MSSQLIterator(body)),
        }
    }
}

pub struct PGSQLIterator<'a>(pub &'a [u8]);
pub struct MSSQLIterator<'a>(pub &'a [u8]);

impl<'a> Iterator for RunningQueriesIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::PGSQL(iter) => iter.next(),
            Self::MSSQL(iter) => iter.next(),
        }
    }
}

impl<'a> Iterator for PGSQLIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<'a> Iterator for MSSQLIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
