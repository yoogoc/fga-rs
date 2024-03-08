use checker::CheckerRef;
use storage::RelationshipTupleReaderRef;

pub struct Handler {
    checker: CheckerRef,
    tuple_reader: RelationshipTupleReaderRef,
}
