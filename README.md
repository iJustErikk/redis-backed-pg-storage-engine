# redis-backed-pg-storage-engine
a fantastically cursed weekend project I need to get to

inspired by phil eaton: https://notes.eatonphil.com/2024-01-09-minimal-in-memory-storage-engine-for-mysql.html

I'll hack this out the weekend of 2/5/2024


## blog steps

- explain storage engines and why this is funny, note this was inspired by phil
- add stubs
- notice that attr macro has compile issues => look at why indexamroutine + zombodb works (search for `unsafe impl SqlTranslatable for crate::IndexAmRoutine`) => figure out you need to third party or make a PR => third party (note that I'll make a PR once I understand this better)
- checkout that third party to `2244e62456390bc10faae99cae3801dc3b05e640`
- keep breaking this until it works