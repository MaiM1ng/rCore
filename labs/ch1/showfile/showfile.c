#include <dirent.h>
#include <stdio.h>
#include <sys/types.h>

int main() {
  char *pathname = ".";
  struct dirent *entry;
  DIR *dir = opendir(pathname);

  if (dir == NULL) {
    printf("Failed to open dir %s\n", pathname);
    return 1;
  }

  while ((entry = readdir((dir)))) {
    printf("%s\n", entry->d_name);
  }

  closedir(dir);

  return 0;
}
