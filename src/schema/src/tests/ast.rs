use crate::*;

#[test]
fn test_relationship_compute_1() {
    let source = Relationship::Union {
        children: vec![
            Box::new(Relationship::Set(RelationshipSet::Single("x".to_string()))),
            Box::new(Relationship::Union {
                children: vec![
                    Box::new(Relationship::Set(RelationshipSet::Single("y".to_string()))),
                    Box::new(Relationship::Union {
                        children: vec![Box::new(Relationship::Set(RelationshipSet::Single("z".to_string())))],
                    }),
                ],
            }),
        ],
    };
    assert_eq!(
        source.compute(),
        Relationship::Union {
            children: vec![
                Box::new(Relationship::Set(RelationshipSet::Single("x".to_string()))),
                Box::new(Relationship::Set(RelationshipSet::Single("y".to_string()))),
                Box::new(Relationship::Set(RelationshipSet::Single("z".to_string()))),
            ]
        }
    )
}

#[test]
fn test_relationship_compute_2() {
    let source = Relationship::Union {
        children: vec![
            Box::new(Relationship::Set(RelationshipSet::Single("x".to_string()))),
            Box::new(Relationship::Union {
                children: vec![
                    Box::new(Relationship::Set(RelationshipSet::Single("y".to_string()))),
                    Box::new(Relationship::Intersection {
                        children: vec![
                            Box::new(Relationship::Set(RelationshipSet::Single("a".to_string()))),
                            Box::new(Relationship::Set(RelationshipSet::Single("b".to_string()))),
                        ],
                    }),
                    Box::new(Relationship::Union {
                        children: vec![Box::new(Relationship::Set(RelationshipSet::Single("z".to_string())))],
                    }),
                ],
            }),
        ],
    };
    assert_eq!(
        source.compute(),
        Relationship::Union {
            children: vec![
                Box::new(Relationship::Set(RelationshipSet::Single("x".to_string()))),
                Box::new(Relationship::Set(RelationshipSet::Single("y".to_string()))),
                Box::new(Relationship::Intersection {
                    children: vec![
                        Box::new(Relationship::Set(RelationshipSet::Single("a".to_string()))),
                        Box::new(Relationship::Set(RelationshipSet::Single("b".to_string()))),
                    ],
                }),
                Box::new(Relationship::Set(RelationshipSet::Single("z".to_string()))),
            ]
        }
    )
}

#[test]
fn test_relationship_compute_3() {
    let source = Relationship::Difference {
        base: Box::new(Relationship::Union {
            children: vec![
                Box::new(Relationship::Set(RelationshipSet::Single("x".to_string()))),
                Box::new(Relationship::Union {
                    children: vec![
                        Box::new(Relationship::Set(RelationshipSet::Single("y".to_string()))),
                        Box::new(Relationship::Intersection {
                            children: vec![
                                Box::new(Relationship::Set(RelationshipSet::Single("a".to_string()))),
                                Box::new(Relationship::Set(RelationshipSet::Single("b".to_string()))),
                            ],
                        }),
                        Box::new(Relationship::Union {
                            children: vec![Box::new(Relationship::Set(RelationshipSet::Single("z".to_string())))],
                        }),
                    ],
                }),
            ],
        }),
        subtract: Box::new(Relationship::Set(RelationshipSet::Single("c".to_string()))),
    };

    assert_eq!(
        source.compute(),
        Relationship::Difference {
            base: Box::new(Relationship::Union {
                children: vec![
                    Box::new(Relationship::Set(RelationshipSet::Single("x".to_string()))),
                    Box::new(Relationship::Set(RelationshipSet::Single("y".to_string()))),
                    Box::new(Relationship::Intersection {
                        children: vec![
                            Box::new(Relationship::Set(RelationshipSet::Single("a".to_string()))),
                            Box::new(Relationship::Set(RelationshipSet::Single("b".to_string()))),
                        ],
                    }),
                    Box::new(Relationship::Set(RelationshipSet::Single("z".to_string()))),
                ]
            }),
            subtract: Box::new(Relationship::Set(RelationshipSet::Single("c".to_string())))
        }
    )
}
