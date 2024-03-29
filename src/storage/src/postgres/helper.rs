use sea_orm::Condition;
use sea_orm::*;

use crate::TupleFilter;

use super::tuple;

pub fn filter_to_conds(filter: &TupleFilter) -> Condition {
    let condition = Condition::all();
    if let Some(object_type_eq) = &filter.object_type_eq {
        condition
            .clone()
            .add(tuple::Column::ObjectType.eq(object_type_eq.to_owned()));
    }
    if let Some(object_id_eq) = &filter.object_id_eq {
        condition
            .clone()
            .add(tuple::Column::ObjectId.eq(object_id_eq.to_owned()));
    }
    if let Some(object_id_in) = &filter.object_id_in {
        condition
            .clone()
            .add(tuple::Column::ObjectId.is_in(object_id_in.to_owned()));
    }
    if let Some(relation_eq) = &filter.relation_eq {
        condition
            .clone()
            .add(tuple::Column::Relation.eq(relation_eq.to_owned()));
    }
    if let Some(user_type_eq) = &filter.user_type_eq {
        condition
            .clone()
            .add(tuple::Column::UserType.eq(user_type_eq.to_owned()));
    }
    if let Some(user_id_eq) = &filter.user_id_eq {
        condition.clone().add(tuple::Column::UserId.eq(user_id_eq.to_owned()));
    }
    if let Some(user_id_in) = &filter.user_id_in {
        condition
            .clone()
            .add(tuple::Column::UserId.is_in(user_id_in.to_owned()));
    }
    if let Some(user_relation_eq) = &filter.user_relation_eq {
        condition
            .clone()
            .add(tuple::Column::UserRelation.eq(user_relation_eq.to_owned()));
    }
    if let Some(or) = &filter.or {
        if !or.is_empty() {
            let or_conds = Condition::any();
            for f in or {
                or_conds.clone().add(filter_to_conds(f));
            }
            condition.clone().add(or_conds);
        }
    }
    condition
}
