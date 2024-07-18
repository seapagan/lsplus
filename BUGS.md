# BUGS

Currently known bugs with this project:

- Very little/no error handling
- if the `path` option is a file, the program will crash. It should show the
file's information instead.
- colorization is not applied to the default short output. fixing this would
need moving the colorization logic to the `collect_file_names` function.
