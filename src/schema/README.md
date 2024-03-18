# Schema

schema dsl such as:
```authz
// comment: model define

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
  relation viewer: user | user#* | group#member | user ^ has_valid_ip
  permission can_change_owner: owner
  permission can_read: viewer + owner + parent#viewer
  permission can_share: owner + parent#owner
  permission can_write: owner + parent#owner
}

// expr eval engine use evalexpr?
// cond has_valid_ip(user_ip ipaddress, allowed_range string) {
//   user_ip.in_cidr(allowed_range)
// }

```
