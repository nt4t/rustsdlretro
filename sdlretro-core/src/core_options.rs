// Libretro core options bindings and management
//
// Supports both v1 (SET_CORE_OPTIONS) and v2 (SET_CORE_OPTIONS_V2) APIs.
// The core provides option definitions via environment callbacks during init.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use super::{
    retro_core_option_value, retro_core_option_definition, retro_core_option_v2_definition,
    RETRO_NUM_CORE_OPTION_VALUES_MAX,
};

/// A single option value (key + display label)
#[derive(Debug, Clone)]
pub struct CoreOptionValue {
    pub value: String,
    pub label: Option<String>,
}

/// A core option with its key, description, available values, and default
#[derive(Debug, Clone)]
pub struct CoreOptionDefinition {
    pub key: String,
    pub desc: String,
    pub info: Option<String>,
    pub values: Vec<CoreOptionValue>,
    pub default_value: Option<String>,
}

/// Category for v2 options (groups related options together)
#[derive(Debug, Clone)]
pub struct CoreOptionCategory {
    pub key: String,
    pub desc: String,
    pub info: Option<String>,
}

/// Complete core options set (v2 format with categories)
#[derive(Debug)]
pub struct CoreOptionsV2 {
    pub categories: Vec<CoreOptionCategory>,
    pub definitions: Vec<CoreOptionDefinition>,
}

/// Complete core options set (v1 format, flat list)
#[derive(Debug)]
pub struct CoreOptionsV1 {
    pub definitions: Vec<CoreOptionDefinition>,
}

/// Parsed core options from the libretro core
#[derive(Debug)]
pub struct CoreOptions {
    /// Whether the core supports v2 API
    pub supports_v2: bool,
    /// The v2 options (if available)
    pub v2: Option<CoreOptionsV2>,
    /// The v1 options (fallback)
    pub v1: Option<CoreOptionsV1>,
}

impl CoreOptions {
    /// Get all option definitions (v2 if available, otherwise v1)
    pub fn definitions(&self) -> Option<&[CoreOptionDefinition]> {
        if self.supports_v2 {
            self.v2.as_ref().map(|v| v.definitions.as_slice())
        } else {
            self.v1.as_ref().map(|v| v.definitions.as_slice())
        }
    }

    /// Get the current value for an option by key
    pub fn get_current_value(&self, key: &str) -> Option<String> {
        // This would be populated from RETRO_ENVIRONMENT_GET_VARIABLE
        // For now, returns the default value
        self.definitions()?.iter().find(|opt| opt.key == key).and_then(|opt| {
            opt.values.iter().find(|v| {
                if let Some(ref default) = opt.default_value {
                    v.value == default
                } else {
                    false
                }
            }).map(|v| v.value.clone())
        })
    }
}

/// Parse a null-terminated array of retro_core_option_value structs
unsafe fn parse_option_values(ptr: *const retro_core_option_value) -> Vec<CoreOptionValue> {
    let mut values = Vec::new();
    if ptr.is_null() {
        return values;
    }

    let mut i = 0;
    loop {
        let val = ptr.add(i);
        let value_ptr = (*val).value;
        let label_ptr = (*val).label;

        // Check for NULL terminator (both value and label are NULL)
        if value_ptr.is_null() && label_ptr.is_null() {
            break;
        }

        if value_ptr.is_null() {
            break;
        }

        let value = CStr::from_ptr(value_ptr).to_string_lossy().into_owned();
        let label = if label_ptr.is_null() {
            None
        } else {
            Some(CStr::from_ptr(label_ptr).to_string_lossy().into_owned())
        };

        values.push(CoreOptionValue { value, label });
        i += 1;

        if i >= RETRO_NUM_CORE_OPTION_VALUES_MAX {
            break;
        }
    }

    values
}

/// Parse a v1 core option definition array
pub unsafe fn parse_v1_definitions(ptr: *const retro_core_option_definition) -> Vec<CoreOptionDefinition> {
    let mut definitions = Vec::new();
    if ptr.is_null() {
        return definitions;
    }

    let mut i = 0;
    loop {
        let def = ptr.add(i);
        let key_ptr = (*def).key;

        // Check for NULL terminator
        if key_ptr.is_null() {
            break;
        }

        let key = CStr::from_ptr(key_ptr).to_string_lossy().into_owned();
        let desc = CStr::from_ptr((*def).desc).to_string_lossy().into_owned();
        let info = if (*def).info.is_null() {
            None
        } else {
            Some(CStr::from_ptr((*def).info).to_string_lossy().into_owned())
        };
        let values = parse_option_values((*def).values.as_ptr());
        let default_value = if (*def).default_value.is_null() {
            None
        } else {
            Some(CStr::from_ptr((*def).default_value).to_string_lossy().into_owned())
        };

        definitions.push(CoreOptionDefinition {
            key,
            desc,
            info,
            values,
            default_value,
        });

        i += 1;
    }

    definitions
}

/// Parse a v2 core option definition array
pub unsafe fn parse_v2_definitions(ptr: *const retro_core_option_v2_definition) -> Vec<CoreOptionDefinition> {
    let mut definitions = Vec::new();
    if ptr.is_null() {
        return definitions;
    }

    let mut i = 0;
    loop {
        let def = ptr.add(i);
        let key_ptr = (*def).key;

        // Check for NULL terminator
        if key_ptr.is_null() {
            break;
        }

        let key = CStr::from_ptr(key_ptr).to_string_lossy().into_owned();
        let desc = CStr::from_ptr((*def).desc).to_string_lossy().into_owned();
        let info = if (*def).info.is_null() {
            None
        } else {
            Some(CStr::from_ptr((*def).info).to_string_lossy().into_owned())
        };
        let values = parse_option_values((*def).values.as_ptr());
        let default_value = if (*def).default_value.is_null() {
            None
        } else {
            Some(CStr::from_ptr((*def).default_value).to_string_lossy().into_owned())
        };

        definitions.push(CoreOptionDefinition {
            key,
            desc,
            info,
            values,
            default_value,
        });

        i += 1;
    }

    definitions
}

/// Parse v2 categories (terminated by zeroed-out category struct)
pub unsafe fn parse_v2_categories(ptr: *mut *mut c_char) -> Vec<CoreOptionCategory> {
    let mut categories = Vec::new();
    if ptr.is_null() {
        return categories;
    }

    // This is a simplified parser - the actual v2 category parsing would need
    // the retro_core_option_v2_category type from bindings
    // For now, return empty categories
    categories
}

/// Parse a complete v2 core options set
pub unsafe fn parse_v2_options(ptr: *mut c_void) -> Option<CoreOptionsV2> {
    if ptr.is_null() {
        return None;
    }

    // The retro_core_options_v2 struct has:
    // - categories: pointer to retro_core_option_v2_category array
    // - definitions: pointer to retro_core_option_v2_definition array
    //
    // We need to parse this from the FFI struct.
    // Note: This requires the actual struct layout from bindgen.
    // For now, we'll parse the definitions directly.

    None
}

/// Check if a core supports the v2 core options API
/// Returns true if GET_CORE_OPTIONS_VERSION returns >= 2
pub fn supports_v2(version: u32) -> bool {
    version >= 2
}

/// Get the v1 core options API version (always 1)
pub const V1_API_VERSION: u32 = 1;

/// Get the v2 core options API version
pub const V2_API_VERSION: u32 = 2;
