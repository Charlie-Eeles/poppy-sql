## Poppy-sql
PostgreSQL formatter for standalone and embedded SQL 

### What is this for?
I like to use raw SQL queries, but maintaining consistent formatting for embedded SQL queries can be quite a pain. 

This project aims to format PostgreSQL queries embedded in a variety of filetypes.

It does this by parsing strings in files and formatting them if they're valid PostgreSQL queries. 

### How to use?
Poppy-sql is available to install through cargo using: `cargo install poppy-sql`

Run `poppy-sql` in a directory with the files you want formatted in it or `poppy-sql --file '{target_file}'` to format a specific file.

### What file types are supported?
At the moment, `.sql` files and `.py` files. 

Other file types will be ignored if poppy-sql is run in the containing folder, or print an error message if specifically targeted.
