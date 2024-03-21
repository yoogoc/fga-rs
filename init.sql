CREATE SEQUENCE global_id_seq;

CREATE OR REPLACE FUNCTION id_generator()
    RETURNS bigint
    LANGUAGE 'plpgsql'
AS $BODY$
DECLARE
    our_epoch bigint := 1314220021721;
    seq_id bigint;
    now_millis bigint;
    -- the id of this DB shard, must be set for each
    -- schema shard you have - you could pass this as a parameter too
    shard_id int := 1;
    result bigint:= 0;
BEGIN
    SELECT nextval('public.global_id_seq') % 1024 INTO seq_id;

    SELECT FLOOR(EXTRACT(EPOCH FROM clock_timestamp()) * 1000) INTO now_millis;
    result := (now_millis - our_epoch) << 23;
    result := result | (shard_id << 10);
    result := result | (seq_id);
	return result;
END;
$BODY$;

CREATE TABLE relation_tuples (
  "id" int8 NOT NULL default id_generator(),
  tenant_id varchar(255),
  user_type varchar(255),
  user_id varchar(255),
  user_relation varchar(255),
  relation varchar(255),
  object_type varchar(255),
  object_id varchar(255),
  created_at timestamp,

  PRIMARY KEY ("id")
);

CREATE TABLE tenants (
  "id" varchar(255),
  name varchar(255),
  created_at timestamp,

  PRIMARY KEY ("id")
);

CREATE TABLE tenants (
  "id" int8 NOT NULL default id_generator(),
  tenant_id varchar(255),
  model jsonb,
  created_at timestamp,

  PRIMARY KEY ("id")
);
