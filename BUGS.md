# BUGS

Currently known bugs with this project:

- Very little/no error handling
- the `-p` option should also add the slash to the '.' and '..' entries, for
consistency with other similar tools.
- if the `path` option is a file, the program will crash. It should show the
file's information instead.
