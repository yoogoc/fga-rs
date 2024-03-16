use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, multispace0, multispace1, newline, space0},
    combinator::{map, recognize},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, tuple},
    IResult,
};

use super::AppError;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
#[allow(dead_code)]
struct SubjectSet<'a> {
    group: &'a str,
    role: &'a str,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
enum Associations<'a> {
    Single(&'a str),
    SubjectSet(SubjectSet<'a>),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
#[allow(dead_code)]
struct Relation<'a> {
    name: &'a str,
    subject: Vec<Associations<'a>>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
#[allow(dead_code)]
struct PermissionSet<'a> {
    group: &'a str,
    permission: &'a str,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
enum Permissions<'a> {
    Single(&'a str),
    PermissionSet(PermissionSet<'a>),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
#[allow(dead_code)]
struct Permission<'a> {
    name: &'a str,
    permissions: Vec<Permissions<'a>>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
#[allow(dead_code)]
struct Definition<'a> {
    name: &'a str,
    relations: Vec<Relation<'a>>,
    permissions: Vec<Permission<'a>>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
#[allow(dead_code)]
pub struct Schema<'a> {
    definitions: Vec<Definition<'a>>,
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(alphanumeric1, many0(pair(char('_'), alphanumeric1))))(input)
}

// usergroup#member
fn parse_entities_associations(input: &str) -> IResult<&str, SubjectSet> {
    let (input, (group, _, role)) = tuple((parse_identifier, tag("#"), parse_identifier))(input)?;
    Ok((input, SubjectSet { group, role }))
}

// usergroup#member OR owner
fn parse_either_entities_associations(input: &str) -> IResult<&str, Associations> {
    alt((
        map(parse_entities_associations, |d| Associations::SubjectSet(d)),
        map(parse_identifier, |d| Associations::Single(d)),
    ))(input)
}

// usergroup#member | usergroup#admin | owner
fn parse_associations(input: &str) -> IResult<&str, Vec<Associations>> {
    let (input, entities) =
        separated_list0(delimited(space0, tag("|"), space0), parse_either_entities_associations)(input)?;

    Ok((input, entities))
}

// relation manager: user | usergroup#member | usergroup#manager
fn parse_relation(input: &str) -> IResult<&str, Relation> {
    let (input, (_, _, _, rel_type, _, _, rel_entities)) = tuple((
        multispace0,
        tag("relation"),
        multispace0,
        parse_identifier,
        tag(":"),
        multispace0,
        parse_associations,
    ))(input)?;

    Ok((
        input,
        Relation {
            name: rel_type,
            subject: rel_entities,
        },
    ))
}

// group->member
fn parse_permission_set(input: &str) -> IResult<&str, PermissionSet> {
    let (input, (group, _, permission)) = tuple((parse_identifier, tag("->"), parse_identifier))(input)?;
    Ok((input, PermissionSet { group, permission }))
}

// administrator OR group->member
fn parse_either_permission(input: &str) -> IResult<&str, Permissions> {
    alt((
        map(parse_permission_set, |ps| Permissions::PermissionSet(ps)),
        map(parse_identifier, |p| Permissions::Single(p)),
    ))(input)
}

// administrator + group->member
fn parse_multi_permissions(input: &str) -> IResult<&str, Vec<Permissions>> {
    let (input, permissions) = separated_list0(delimited(space0, tag("+"), space0), parse_either_permission)(input)?;

    Ok((input, permissions))
}

// permission member = direct_member + manager
fn parse_permission(input: &str) -> IResult<&str, Permission> {
    let (input, (_, _, _, permission, _, _, _, permissions)) = tuple((
        multispace0,
        tag("permission"),
        multispace1,
        parse_identifier,
        multispace0,
        tag("="),
        multispace0,
        parse_multi_permissions,
    ))(input)?;

    Ok((
        input,
        Permission {
            name: permission,
            permissions,
        },
    ))
}

fn parse_all_relations(input: &str) -> IResult<&str, Vec<Relation>> {
    separated_list0(newline, parse_relation)(input)
}

fn parse_all_permissions(input: &str) -> IResult<&str, Vec<Permission>> {
    separated_list0(newline, parse_permission)(input)
}

fn parse_definition(input: &str) -> IResult<&str, Definition> {
    let (input, (_, definition, _, relations, _, permissions, _, _, _)) = tuple((
        tuple((multispace0, tag("definition"), multispace0)),
        parse_identifier,
        tuple((multispace0, tag("{"), multispace0)),
        parse_all_relations,
        multispace0,
        parse_all_permissions,
        multispace0,
        char('}'),
        multispace0,
    ))(input)?;

    Ok((
        input,
        Definition {
            name: definition,
            relations,
            permissions,
        },
    ))
}
fn parse_schema_definitions(input: &str) -> IResult<&str, Schema> {
    let (input, definitions) = many0(delimited(multispace0, parse_definition, multispace0))(input)?;

    Ok((input, Schema { definitions }))
}

pub fn parse_schema(input: &str) -> Result<Schema, AppError> {
    let (_, schema) = parse_schema_definitions(input).map_err(|_| AppError::ParsingError)?;

    if schema.definitions.is_empty() {
        return Err(AppError::InvalidSchema);
    }
    Ok(schema)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_entities_associations() {
        let association = SubjectSet {
            group: "usergroup",
            role: "member",
        };
        let result = dbg!(parse_entities_associations("usergroup#member"));

        let Ok((_, parsed_association)) = result else {
            panic!();
        };
        assert_eq!(association, parsed_association);
    }

    #[test]
    fn test_parse_multi_associations() {
        let association = vec![
            Associations::SubjectSet(SubjectSet {
                group: "usergroup",
                role: "member",
            }),
            Associations::SubjectSet(SubjectSet {
                group: "usergroup",
                role: "admin",
            }),
            Associations::Single("owner"),
        ];
        let result = dbg!(parse_associations("usergroup#member | usergroup#admin | owner"));

        let Ok((_, parsed_association)) = result else {
            panic!();
        };
        assert_eq!(association, parsed_association);
    }

    #[test]
    fn test_parse_relation() {
        let role = "       relation manager: user | usergroup#member | usergroup#manager";
        let relation = Relation {
            name: "manager",
            subject: vec![
                Associations::Single("user"),
                Associations::SubjectSet(SubjectSet {
                    group: "usergroup",
                    role: "member",
                }),
                Associations::SubjectSet(SubjectSet {
                    group: "usergroup",
                    role: "manager",
                }),
            ],
        };
        let result = dbg!(parse_relation(dbg!(role)));

        let Ok((_, parsed)) = result else {
            panic!();
        };
        assert_eq!(relation, parsed);
    }

    #[test]
    fn test_parse_multi_relation() {
        let s = "       relation manager: user | usergroup#member | usergroup#manager
               relation manager: user | usergroup#member | usergroup#manager";
        let p = vec![
            Relation {
                name: "manager",
                subject: vec![
                    Associations::Single("user"),
                    Associations::SubjectSet(SubjectSet {
                        group: "usergroup",
                        role: "member",
                    }),
                    Associations::SubjectSet(SubjectSet {
                        group: "usergroup",
                        role: "manager",
                    }),
                ],
            },
            Relation {
                name: "manager",
                subject: vec![
                    Associations::Single("user"),
                    Associations::SubjectSet(SubjectSet {
                        group: "usergroup",
                        role: "member",
                    }),
                    Associations::SubjectSet(SubjectSet {
                        group: "usergroup",
                        role: "manager",
                    }),
                ],
            },
        ];
        let result = dbg!(parse_all_relations(dbg!(s)));

        let Ok((_, parsed)) = result else {
            panic!();
        };
        assert_eq!(p, parsed);
    }

    #[test]
    fn test_parse_permission_set() {
        let permission_set = PermissionSet {
            group: "group",
            permission: "member",
        };
        let result = dbg!(parse_permission_set("group->member"));

        let Ok((_, parsed)) = result else {
            panic!();
        };
        assert_eq!(permission_set, parsed);
    }

    #[test]
    fn test_parse_multi_permissions() {
        let multi_permissions = vec![
            Permissions::Single("direct_member"),
            Permissions::Single("manager"),
            Permissions::PermissionSet(PermissionSet {
                group: "group",
                permission: "member",
            }),
        ];
        let result = dbg!(parse_multi_permissions("direct_member + manager + group->member"));

        let Ok((_, parsed)) = result else {
            panic!();
        };
        assert_eq!(multi_permissions, parsed);
    }

    #[test]
    fn test_parse_permission() {
        let permission = Permission {
            name: "member",
            permissions: vec![
                Permissions::Single("direct_member"),
                Permissions::Single("administrator"),
                Permissions::PermissionSet(PermissionSet {
                    group: "group",
                    permission: "member",
                }),
            ],
        };
        let result = dbg!(parse_permission(
            "     permission member = direct_member + administrator + group->member"
        ));

        let Ok((_, parsed)) = result else {
            panic!();
        };
        assert_eq!(permission, parsed);
    }

    #[test]
    fn test_parse_multi_permission() {
        let permission_str = "     permission member = direct_member + administrator + group->member
        permission member = direct_member + administrator + group->member";
        let pers = vec![
            Permission {
                name: "member",
                permissions: vec![
                    Permissions::Single("direct_member"),
                    Permissions::Single("administrator"),
                    Permissions::PermissionSet(PermissionSet {
                        group: "group",
                        permission: "member",
                    }),
                ],
            },
            Permission {
                name: "member",
                permissions: vec![
                    Permissions::Single("direct_member"),
                    Permissions::Single("administrator"),
                    Permissions::PermissionSet(PermissionSet {
                        group: "group",
                        permission: "member",
                    }),
                ],
            },
        ];
        let result = dbg!(parse_all_permissions(dbg!(permission_str)));

        let Ok((_, parsed)) = result else {
            panic!();
        };
        assert_eq!(pers, parsed);
    }

    #[test]
    fn test_parse_empty_definition() {
        let definition_str = "definition user {}";
        let definition = Definition {
            name: "user",
            relations: vec![],
            permissions: vec![],
        };
        let result = dbg!(parse_definition(dbg!(definition_str)));

        let Ok((_, parsed)) = result else {
            panic!();
        };
        assert_eq!(definition, parsed);
    }

    #[test]
    fn test_parse_definition() {
        let definition_str = "
        definition resource {
                    relation manager: user | usergroup#member | usergroup#manager
                    relation viewer: user | usergroup#member | usergroup#manager

                    permission manage = manager
                    permission view = viewer + manager
                }";
        let definition = Definition {
            name: "resource",
            relations: vec![
                Relation {
                    name: "manager",
                    subject: vec![
                        Associations::Single("user"),
                        Associations::SubjectSet(SubjectSet {
                            group: "usergroup",
                            role: "member",
                        }),
                        Associations::SubjectSet(SubjectSet {
                            group: "usergroup",
                            role: "manager",
                        }),
                    ],
                },
                Relation {
                    name: "viewer",
                    subject: vec![
                        Associations::Single("user"),
                        Associations::SubjectSet(SubjectSet {
                            group: "usergroup",
                            role: "member",
                        }),
                        Associations::SubjectSet(SubjectSet {
                            group: "usergroup",
                            role: "manager",
                        }),
                    ],
                },
            ],
            permissions: vec![
                Permission {
                    name: "manage",
                    permissions: vec![Permissions::Single("manager")],
                },
                Permission {
                    name: "view",
                    permissions: vec![Permissions::Single("viewer"), Permissions::Single("manager")],
                },
            ],
        };
        let result = dbg!(parse_definition(dbg!(definition_str)));

        let Ok((_, parsed)) = result else {
            panic!();
        };
        assert_eq!(definition, parsed);
    }

    #[test]
    fn test_parse_schema() {
        let s = "
        definition user {}

        definition resource {
            relation manager: user | usergroup#member | usergroup#manager
            permission view = viewer + manager
        }";
        let p = Schema {
            definitions: vec![
                Definition {
                    name: "user",
                    relations: vec![],
                    permissions: vec![],
                },
                Definition {
                    name: "resource",
                    relations: vec![Relation {
                        name: "manager",
                        subject: vec![
                            Associations::Single("user"),
                            Associations::SubjectSet(SubjectSet {
                                group: "usergroup",
                                role: "member",
                            }),
                            Associations::SubjectSet(SubjectSet {
                                group: "usergroup",
                                role: "manager",
                            }),
                        ],
                    }],
                    permissions: vec![Permission {
                        name: "view",
                        permissions: vec![Permissions::Single("viewer"), Permissions::Single("manager")],
                    }],
                },
            ],
        };
        let result = dbg!(parse_schema(dbg!(s)));

        let Ok(parsed) = result else {
            panic!();
        };
        assert_eq!(p, parsed);
    }

    #[test]
    fn test_parse_invalid_schema() {
        let s = r#"
        schema {
            query: Query
        }
        
        type Query {
            hello: String
        }
        "#;

        let result = dbg!(parse_schema(dbg!(s)));

        assert!(result.is_err());
    }
}
