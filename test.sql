DROP EXTENSION IF EXISTS redis_backed_storage CASCADE;
CREATE EXTENSION redis_backed_storage;
CREATE TABLE x(a INT) USING redis;
SELECT a FROM x;