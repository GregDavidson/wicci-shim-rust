# Wicci Shim

An HTTP reverse-proxy server which bridges between a client's browser
and a PostgreSQL-based Wicci Web Server.  A key component of the Wicci
Web Framework.

This is a new implementation in Rust.  I have written other Wicci
Shims using other techniques and software stacks.  My most stable and
complete Shim is written in C for a Posix environment.  Although this
new implementation is not quite done it will soon obsolete the others.
Posix system. Both of these implement smart database connection pooling.
