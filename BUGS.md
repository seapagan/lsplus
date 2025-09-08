# BUGS

Currently known bugs with this project:

- Very little/no error handling
- colorization is not applied to the default short output. fixing this would
need moving the colorization logic to the `collect_file_names` function.
- if more than one path is provided they should have a gap between them, and a
header for each.
- wild-cards for the path are not supported and will cause the program to crash.
- when sorting, dot files should come before non-dot files with the same sorting
ie `.changelog` should come before `changelog` etc.
- In the short-form layout, every file after a link (postfix with a star) on the
  same line is out of alignment by one space, cumulative.
