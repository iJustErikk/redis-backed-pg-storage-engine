# redis-backed-pg-storage-engine
a fantastically cursed weekend project I need to get to

inspired by phil eaton: https://notes.eatonphil.com/2024-01-09-minimal-in-memory-storage-engine-for-mysql.html

I'll hack this out the weekend of 2/5/2024

## thoughts

what are we going to define as "redis-backed"?

storing absolutely all state in redis would be a massive PINA. if these get out of sync (command is successful yet we don't get response), then we might "lie".

more on this:
they issue create table
we call redis and create that table in our redis repr, yet we don't get the response
we say that failed
they issue some operation on that table (insert/select)
we properly reject those for the table not existing
... server restarts and rebootstraps redis state
they issue insert/select and it successfully goes through (it shouldn't- table creation was not successful)

my solution to this is to not do this.

not having transactions is fine since we only have 1 redis write per insert/create table

## design decisions

will everything be a redis command? or will we store some state?

we need to store scan state, no way around that (postgres implements scans as an iterator)

everything else can be in redis

CREATE TABLE:

insert (table name, anything lol)

INSERT:

GET table_name => insert (table name + random numbers as key, value)

SELECT:

prefix scan with table name prefix (skip over keys == table name (table marker))

we need to store this iterator for getnextslot^, just store in map (scan id -> redis scan result iterator)


## what will we support?

this will be incredibly limited.

the only datatype will be string and you can only use one column!

CREATE TABLE, SELECT (any bytea columns), INSERT

100% does not support concurrent requests (though we could use redis locks lol)

connection issues with redis may lead to incorrect results or corrupted state

does not write-ahead

obviously does not survive redis restart


## blog steps

- explain storage engines and why this is funny, note this was inspired by phil
- add stubs
- notice that attr macro has compile issues => look at why indexamroutine + zombodb works (search for `unsafe impl SqlTranslatable for crate::IndexAmRoutine`) => figure out you need to third party or make a PR => third party (note that I'll make a PR once I understand this better)
- checkout that third party to `2244e62456390bc10faae99cae3801dc3b05e640`
- keep breaking this until it works