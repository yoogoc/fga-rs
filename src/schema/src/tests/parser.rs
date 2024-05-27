use crate::*;

#[test]
fn test_lexer() {
    let schema = r"// comment: model define
// define user type, no relation, no permission
type user {}

type block {
  relation assignment: user
}
";
    let mut comments = vec![];
    let tokens = Lexer::new(schema, &mut comments).collect::<Vec<_>>();

    assert_eq!(
        tokens,
        vec![
            Ok((73, Token::Type, 77)),
            Ok((78, Token::Identifier("user"), 82)),
            Ok((83, Token::LBrace, 84)),
            Ok((84, Token::RBrace, 85)),
            Ok((85, Token::Newline, 86)),
            // Ok((86, Token::Newline, 87)),
            Ok((87, Token::Type, 91)),
            Ok((92, Token::Identifier("block"), 97)),
            Ok((98, Token::LBrace, 99)),
            Ok((99, Token::Newline, 100)),
            Ok((102, Token::Relation, 110)),
            Ok((111, Token::Identifier("assignment"), 121)),
            Ok((121, Token::Colon, 122)),
            Ok((123, Token::Identifier("user"), 127)),
            Ok((127, Token::Newline, 128)),
            Ok((128, Token::RBrace, 129)),
            Ok((129, Token::Newline, 130)),
        ]
    );

    assert_eq!(
        comments,
        vec![
            ((0, 24), String::from("// comment: model define")),
            (
                (25, 72),
                String::from("// define user type, no relation, no permission")
            ),
        ]
    );
}

#[test]
fn test_parser1() {
    let schema = r"// comment: model define
// define user type, no relation, no permission
type user {}

type block {
  relation assignment: user
}
";
    let result = parse(schema).unwrap();
    assert_eq!(
        result.0.types,
        vec![
            Type {
                name: String::from("user"),
                relations: vec![],
                permissions: vec![]
            },
            Type {
                name: String::from("block"),
                relations: vec![Relation {
                    name: String::from("assignment"),
                    subjects: vec![RelationshipSet::Single(String::from("user"))]
                }],
                permissions: vec![]
            }
        ]
    )
}

#[test]
fn test_parser2() {
    let schema = r"// comment: model define
// define user type, no relation, no permission
type user {}

type block {
  relation assignment: user
}

// define group type, has a relation
type group {
  relation member: user
}

// define group type, has some relations and some permissions
type folder {
  relation owner: user
  relation parent: folder
  relation viewer: user | user#* | group#member | owner | parent#viewer
}";
    let result = parse(schema).unwrap();
    assert_eq!(
        result.0.types,
        vec![
            Type {
                name: String::from("user"),
                relations: vec![],
                permissions: vec![]
            },
            Type {
                name: String::from("block"),
                relations: vec![Relation {
                    name: String::from("assignment"),
                    subjects: vec![RelationshipSet::Single(String::from("user"))]
                }],
                permissions: vec![]
            },
            Type {
                name: String::from("group"),
                relations: vec![Relation {
                    name: String::from("member"),
                    subjects: vec![RelationshipSet::Single(String::from("user"))]
                }],
                permissions: vec![]
            },
            Type {
                name: String::from("folder"),
                relations: vec![
                    Relation {
                        name: String::from("owner"),
                        subjects: vec![RelationshipSet::Single(String::from("user"))]
                    },
                    Relation {
                        name: String::from("parent"),
                        subjects: vec![RelationshipSet::Single(String::from("folder"))]
                    },
                    Relation {
                        name: String::from("viewer"),
                        subjects: vec![
                            RelationshipSet::Single(String::from("user")),
                            RelationshipSet::Set(String::from("user"), String::from("*")),
                            RelationshipSet::Set(String::from("group"), String::from("member")),
                            RelationshipSet::Single(String::from("owner")),
                            RelationshipSet::Set(String::from("parent"), String::from("viewer")),
                        ]
                    }
                ],
                permissions: vec![]
            }
        ]
    )
}

#[test]
fn test_parser3() {
    let schema = r"// comment: model define

    // define user type, no relation, no permission
    type user {}

    type block {
      relation assignment: user
    }

    // define group type, has a relation
    type group {
      relation member: user
    }

    // define group type, has some relations and some permissions
    type folder {
      relation owner: user
      relation parent: folder
      relation viewer: user | user#* | group#member | owner | parent#viewer
      permission view: viewer - block#assignment
    }";
    let result = parse(schema).unwrap();
    assert_eq!(
        result.0.types,
        vec![
            Type {
                name: String::from("user"),
                relations: vec![],
                permissions: vec![]
            },
            Type {
                name: String::from("block"),
                relations: vec![Relation {
                    name: String::from("assignment"),
                    subjects: vec![RelationshipSet::Single(String::from("user"))]
                }],
                permissions: vec![]
            },
            Type {
                name: String::from("group"),
                relations: vec![Relation {
                    name: String::from("member"),
                    subjects: vec![RelationshipSet::Single(String::from("user"))]
                }],
                permissions: vec![]
            },
            Type {
                name: String::from("folder"),
                relations: vec![
                    Relation {
                        name: String::from("owner"),
                        subjects: vec![RelationshipSet::Single(String::from("user"))]
                    },
                    Relation {
                        name: String::from("parent"),
                        subjects: vec![RelationshipSet::Single(String::from("folder"))]
                    },
                    Relation {
                        name: String::from("viewer"),
                        subjects: vec![
                            RelationshipSet::Single(String::from("user")),
                            RelationshipSet::Set(String::from("user"), String::from("*")),
                            RelationshipSet::Set(String::from("group"), String::from("member")),
                            RelationshipSet::Single(String::from("owner")),
                            RelationshipSet::Set(String::from("parent"), String::from("viewer")),
                        ]
                    },
                ],
                permissions: vec![Permission {
                    name: String::from("view"),
                    permission: Relationship::Difference {
                        base: Box::new(Relationship::Set(RelationshipSet::Single(String::from("viewer")))),
                        subtract: Box::new(Relationship::Set(RelationshipSet::Set(
                            String::from("block"),
                            String::from("assignment")
                        ))),
                    }
                }]
            }
        ]
    )
}

#[test]
fn test_parser4() {
    let schema = r"// comment: model define

    // define user type, no relation, no permission
    type user {}

    type block {
      relation assignment: user
    }

    // define group type, has a relation
    type group {
      relation member: user
    }

    // define group type, has some relations and some permissions
    type folder {
      relation owner: user
      relation parent: folder
      relation viewer: user | user#* | group#member | owner | parent#viewer
      permission view: viewer - block#assignment
    }

    // define group type, has some relations and some permissions
    type doc {
      relation owner: user
      relation parent: folder
      relation viewer: user | user#* | group#member // | user ^ has_valid_ip
      permission can_change_owner: owner
      permission can_read: viewer + owner + parent#viewer
      permission can_share: owner + parent#owner
      permission can_write: owner + parent#owner
    }

    // expr eval engine use evalexpr?
    condition has_valid_ip(user_ip: ipaddress, allowed_range: string) {
      user_ip.in_cidr(allowed_range)
    }";
    let result = parse(schema).unwrap();
    assert_eq!(
        result.0.types,
        vec![
            Type {
                name: String::from("user"),
                relations: vec![],
                permissions: vec![]
            },
            Type {
                name: String::from("block"),
                relations: vec![Relation {
                    name: String::from("assignment"),
                    subjects: vec![RelationshipSet::Single(String::from("user"))]
                }],
                permissions: vec![]
            },
            Type {
                name: String::from("group"),
                relations: vec![Relation {
                    name: String::from("member"),
                    subjects: vec![RelationshipSet::Single(String::from("user"))]
                }],
                permissions: vec![]
            },
            Type {
                name: String::from("folder"),
                relations: vec![
                    Relation {
                        name: String::from("owner"),
                        subjects: vec![RelationshipSet::Single(String::from("user"))]
                    },
                    Relation {
                        name: String::from("parent"),
                        subjects: vec![RelationshipSet::Single(String::from("folder"))]
                    },
                    Relation {
                        name: String::from("viewer"),
                        subjects: vec![
                            RelationshipSet::Single(String::from("user")),
                            RelationshipSet::Set(String::from("user"), String::from("*")),
                            RelationshipSet::Set(String::from("group"), String::from("member")),
                            RelationshipSet::Single(String::from("owner")),
                            RelationshipSet::Set(String::from("parent"), String::from("viewer")),
                        ]
                    },
                ],
                permissions: vec![Permission {
                    name: String::from("view"),
                    permission: Relationship::Difference {
                        base: Box::new(Relationship::Set(RelationshipSet::Single(String::from("viewer")))),
                        subtract: Box::new(Relationship::Set(RelationshipSet::Set(
                            String::from("block"),
                            String::from("assignment")
                        ))),
                    }
                }]
            },
            Type {
                name: String::from("doc"),
                relations: vec![
                    Relation {
                        name: String::from("owner"),
                        subjects: vec![RelationshipSet::Single(String::from("user"))]
                    },
                    Relation {
                        name: String::from("parent"),
                        subjects: vec![RelationshipSet::Single(String::from("folder"))]
                    },
                    Relation {
                        name: String::from("viewer"),
                        subjects: vec![
                            RelationshipSet::Single(String::from("user")),
                            RelationshipSet::Set(String::from("user"), String::from("*")),
                            RelationshipSet::Set(String::from("group"), String::from("member")),
                        ]
                    },
                ],
                permissions: vec![
                    Permission {
                        name: String::from("can_change_owner"),
                        permission: Relationship::Set(RelationshipSet::Single(String::from("owner")))
                    },
                    Permission {
                        name: String::from("can_read"),
                        permission: Relationship::Union {
                            children: vec![
                                Box::new(Relationship::Set(RelationshipSet::Single(String::from("viewer")))),
                                Box::new(Relationship::Set(RelationshipSet::Single(String::from("owner")))),
                                Box::new(Relationship::Set(RelationshipSet::Set(
                                    String::from("parent"),
                                    String::from("viewer")
                                ))),
                            ]
                        }
                    },
                    Permission {
                        name: String::from("can_share"),
                        permission: Relationship::Union {
                            children: vec![
                                Box::new(Relationship::Set(RelationshipSet::Single(String::from("owner")))),
                                Box::new(Relationship::Set(RelationshipSet::Set(
                                    String::from("parent"),
                                    String::from("owner")
                                ))),
                            ]
                        }
                    },
                    Permission {
                        name: String::from("can_write"),
                        permission: Relationship::Union {
                            children: vec![
                                Box::new(Relationship::Set(RelationshipSet::Single(String::from("owner")))),
                                Box::new(Relationship::Set(RelationshipSet::Set(
                                    String::from("parent"),
                                    String::from("owner")
                                ))),
                            ]
                        }
                    },
                ]
            }
        ]
    )
}
