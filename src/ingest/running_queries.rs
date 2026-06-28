use crate::parser::running_queries;

pub enum RunningQueriesIterator<'a> {
    PGSQL(PGSQLIterator<'a>),
    MSSQL(MSSQLIterator<'a>),
}

impl<'a> RunningQueriesIterator<'a> {
    pub fn new(strategy: running_queries::Strategy, body: &'a [u8]) -> Self {
        match strategy {
            running_queries::Strategy::PGSQL => RunningQueriesIterator::PGSQL(PGSQLIterator(body)),
            running_queries::Strategy::MSSQL => RunningQueriesIterator::MSSQL(MSSQLIterator(body)),
        }
    }
}

struct PGSQLIterator<'a>(&'a [u8]);
struct MSSQLIterator<'a>(&'a [u8]);

impl<'a> Iterator for RunningQueriesIterator<'a> {
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
