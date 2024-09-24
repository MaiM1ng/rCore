#define UNW_LOCAL_ONLY
#include <libunwind.h>
#include <stdio.h>

void print_func_strace() {
  unw_cursor_t cursor;
  unw_context_t context;
  char sym[128];
  unw_word_t offset;

  unw_getcontext(&context);
  unw_init_local(&cursor, &context);

  printf("---------backstrace---------\n");

  while (unw_step(&cursor) > 0) {
    unw_get_proc_name(&cursor, sym, sizeof(sym), &offset);

    printf("Func: %s\n", sym);
  }
}

void func1() { print_func_strace(); }

void func2() { func1(); }

int main() {
  func2();

  return 0;
}
