use criterion::{black_box, criterion_group, criterion_main, Criterion};
use schema::parse_schema;

pub fn criterion_benchmark(c: &mut Criterion) {
    let input = r#"
    definition user {}

    definition resource {
        relation manager: user | usergroup#member | usergroup#manager
        relation viewer: user | usergroup#member | usergroup#manager
    
        permission manage = manager
        permission view = viewer + manager
    }
    
    definition usergroup {
        relation manager: user | usergroup#member | usergroup#manager
        relation direct_member: user | usergroup#member | usergroup#manager
    
        permission member = direct_member + manager
    }
    
    definition organization {
        relation group: usergroup
        relation administrator: user | usergroup#member | usergroup#manager
        relation direct_member: user
    
        relation resource: resource
    
        permission admin = administrator
        permission member = direct_member + administrator + group->member
    }
    "#;

    c.bench_function("parse_s", |b| b.iter(|| parse_schema(black_box(&input))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
