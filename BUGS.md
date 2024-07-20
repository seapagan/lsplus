# BUGS

Currently known bugs with this project:

- Very little/no error handling
- colorization is not applied to the default short output. fixing this would
need moving the colorization logic to the `collect_file_names` function.
- if more than one path is provided the program crashes. `ls` and other
derivatives will show both
- wild-cards for the path are not supported and will cause the program to crash.
