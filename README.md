## Poppy
PostgreSQL formatter for standalone and embedded SQL 

### What is this for?
I like to use raw SQL queries, but maintaining consistent formatting for embedded SQL queries can be quite a pain. 

This project aims to format PostgreSQL queries embedded in a variety of filetypes.

### Is it production ready?
No. This is a very new project and its parsing is rudimentary. Check back later or take the risk if you'd like, just make sure your files are in version control.

### How to use?
Poppy is available to install through cargo using: `cargo install poppy-sql`

Run `poppy` in a directory with the files you want formatted in it or `poppy --file '{target_file}'` to format a specific file.
