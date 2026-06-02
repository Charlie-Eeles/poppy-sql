## Acknowledgments

This formatting module is based largely on the hard work done on the sqlformat-rs library and by extension sql-formatter-plus that it was based on.

*Links*

https://github.com/shssoichiro/sqlformat-rs

https://github.com/kufii/sql-formatter-plus

The decision to integrate the code directly rather than depend on a brilliant external library was made because:

- Formatting is so integral to the functionality of Poppy and SQL formatting is very opinionated, it's important that Poppy is able to reflect those opinions without them being pressured onto another library.
- Poppy's focus on embedded SQL makes it so there are some other operations that would be more efficient to integrate into the same formatting stage to avoid double processing.
- It's intended for Poppy to become a linter in addition to a formatter, and for that it's more efficient to do that analysis at the same stage as formatting.

Poppy is not aiming to be a direct "competitor" of sqlformat-rs, Poppy's library is more focused on parsing than formatting and is primarily intended to be used as a binary.\
It does publicly expose the formatting module as part of the library but mostly as a matter of convenience, it's not the foremost focus of this package.

If you need a formatting library I encourage you to make use of sqlformat-rs, if you do decide to use Poppy as your formatting library, then I hope I've made it clear we didn't do this alone :-)
