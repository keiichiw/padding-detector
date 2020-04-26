/* A library to be inserted to an auto-generated code */

/// Updates a given layout with a field and check if a padding is added.
fn extend_layout<T>(l: &std::alloc::Layout, name: &str, v: &T) -> std::alloc::Layout {
    let (new_l, offset) = l.extend(std::alloc::Layout::for_value(v)).expect("x");
    if offset != l.size() {
        println!("{}-byte padding before \"{}\"", offset - l.size(), name);
    }
    new_l
}

/// Calls `extend_layout` with multiple fields.
macro_rules! add_field {
    ($layout:ident, $strct:ident, $id:ident) => {
        $layout = extend_layout(&$layout, stringify!($id), &$strct.$id);
    };
    ($layout:ident, $strct:ident, $id:ident $(, $more:ident)+ ) => {
        add_field!($layout, $strct, $id);
        add_field!($layout, $strct  $(, $more)+);
    };
}

/// Checks struct paddings by check_struct!(<struct name>, <field name>,...).
macro_rules! check_struct {
    ($strct:ty, $( $field:ident ),+ ) => {
        {
            println!("Checking `struct {}`...", stringify!($strct));
            let instance: $strct = Default::default();
            let mut layout = std::alloc::Layout::from_size_align(0, 1).unwrap();

            // Update `layout` by extending with fields.
            add_field!(layout, instance, $($field),+);

            // Check if a padding will be inserted at the end of struct.
            let pad = layout.padding_needed_for(layout.align());
            if pad != 0 {
                println!("{}-byte padding at the end", pad);
            }
            layout = layout.pad_to_align();
            assert_eq!(layout.size(), std::mem::size_of_val(&instance));
        }
    };
}

/// Calculates the max size of fields in a given union instance.
macro_rules! max_field_size {
    ($instance:ident, $field:ident) => {
        unsafe { std::mem::size_of_val(&$instance.$field) }
    };
    ($instance:ident, $field:ident, $($more:ident),+) => {
        std::cmp::max(max_field_size!($instance, $field), max_field_size!($instance, $($more),+))
    }
}

/// Checks union's padding by check_union!(<union name>, <field name>,...).
macro_rules! check_union {
    ($union:ty, $( $field:ident ),+ ) => {{
        println!("Checking `union {}`...", stringify!($union));
        let instance: $union = Default::default();
        let max_size = max_field_size!(instance, $($field),+);
        let diff = std::mem::size_of_val(&instance) - max_size;
        if diff != 0 {
            println!("{}-byte padding is inserted", diff);
        }
    }};
}
