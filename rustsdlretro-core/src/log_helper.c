#include <stdarg.h>
#include <stdio.h>

/* C helper that expands variadic format string and calls Rust handler.
 * This is called from the non-variadic log_callback via dlsym(RTLD_DEFAULT). */

extern void rust_log_callback_va(unsigned level, const char* fmt, va_list ap);

void __attribute__((visibility("default"))) rustsdlretro_log_handler(unsigned level, const char* fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    rust_log_callback_va(level, fmt, ap);
    va_end(ap);
}
