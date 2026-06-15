# Core Options

## Architecture

Three layers:

1. **libretro.h** — defines the standard libretro structs:
   - `retro_core_option_value` — `{ value, label }` pair for each option choice
   - `retro_core_option_definition` — `{ key, desc, info, values[], default_value }`
   - `retro_core_options_intl` — US + localized option arrays
   - `retro_core_option_v2_definition` — extended v2 with category support

2. **variables.h/cpp** — `retro_variables` class that wraps and manages options:
   - `retro_variable_t` — runtime struct with `name`, `curr_index`, `default_index`, `label`, `info`, `visible`, `options[]`
   - `load_variables()` — parses `retro_core_option_definition[]` or `retro_variable[]` arrays, matches default values, builds option map
   - `load_variables_from_cfg()` / `save_variables_to_cfg()` — persists options as JSON using option `value` strings as keys
   - `get_variable()` / `set_variable()` — runtime get/set by key

3. **driver_base.cpp** — `env_callback()` handles libretro environment commands:
   - `RETRO_ENVIRONMENT_SET_CORE_OPTIONS` → calls `variables->load_variables()` + loads from JSON config
   - `RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL` → loads US or localized array based on language
   - `RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY` → toggles visibility of variables
   - `RETRO_ENVIRONMENT_GET_VARIABLE` → returns the current value string for a given key
   - `RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE` → returns whether any variable changed since last check

## Config Storage

Options are stored per-core in `{store_dir}/cfg/cores/{CoreName}.json` as a flat JSON object mapping option keys to their selected value strings. On load, the JSON is parsed and matching option indices are set. On unload, `save_variables_to_cfg()` writes the current selection back.

## Usage Pattern (for a core)

```cpp
static const struct retro_core_option_value some_option_values[] = {
    { "disabled", NULL },
    { "enabled",  NULL },
    { NULL, NULL }
};

static struct retro_core_option_definition some_option = {
    "mycore_some_option",
    "Some Option",
    "Description shown as sublabel",
    some_option_values,
    "disabled"
};

static struct retro_core_option_definition *options[] = { &some_option, NULL };

// In retro_set_environment():
environ_cb(RETRO_ENVIRONMENT_SET_CORE_OPTIONS, options);
```

The frontend then queries values via `RETRO_ENVIRONMENT_GET_VARIABLE` with the `key` string (`"mycore_some_option"`).

sources: E:\LLM\src\sdlretro\src 