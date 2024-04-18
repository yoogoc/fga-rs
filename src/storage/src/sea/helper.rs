use sea_orm::Condition;
use sea_orm::*;

use crate::TupleFilter;

use super::tuple;

pub fn filter_to_conds(filter: &TupleFilter) -> Condition {
    let mut condition = Condition::all();
    if let Some(object_type_eq) = &filter.object_type_eq {
        condition = condition.add(tuple::Column::ObjectType.eq(object_type_eq.to_owned()));
    }
    if let Some(object_id_eq) = &filter.object_id_eq {
        condition = condition.add(tuple::Column::ObjectId.eq(object_id_eq.to_owned()));
    }
    if let Some(object_id_in) = &filter.object_id_in {
        condition = condition.add(tuple::Column::ObjectId.is_in(object_id_in.to_owned()));
    }
    if let Some(relation_eq) = &filter.relation_eq {
        condition = condition.add(tuple::Column::Relation.eq(relation_eq.to_owned()));
    }
    if let Some(user_type_eq) = &filter.user_type_eq {
        condition = condition.add(tuple::Column::UserType.eq(user_type_eq.to_owned()));
    }
    if let Some(user_id_eq) = &filter.user_id_eq {
        condition = condition.add(tuple::Column::UserId.eq(user_id_eq.to_owned()));
    }
    if let Some(user_id_in) = &filter.user_id_in {
        condition = condition.add(tuple::Column::UserId.is_in(user_id_in.to_owned()));
    }
    if let Some(user_relation_eq) = &filter.user_relation_eq {
        condition = condition.add(tuple::Column::UserRelation.eq(user_relation_eq.to_owned()));
    }
    if let Some(user_relation_is_null) = &filter.user_relation_is_null {
        if user_relation_is_null.to_owned() {
            condition = condition.add(tuple::Column::UserRelation.is_null());
        } else {
            condition = condition.add(tuple::Column::UserRelation.is_not_null());
        }
    }
    if let Some(or) = &filter.or {
        if !or.is_empty() {
            let mut or_conds = Condition::any();
            for f in or {
                or_conds = or_conds.add(filter_to_conds(f));
            }
            condition = condition.add(or_conds);
        }
    }
    condition
}
