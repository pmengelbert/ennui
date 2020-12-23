#![feature(try_trait)]
#![feature(assoc_char_funcs)]

#[macro_export]
macro_rules! arc_mutex(
    ($wrapped:expr) => {
        Arc::new(Mutex::new($wrapped))
    };
);

#[macro_export]
macro_rules! goto_cleanup_on_fail {
    ($res:expr, $label:tt) => {
        match $res {
            Some(r) => r,
            None => break $label,
        }
    };
}

#[macro_export]
/// Wrap a block of fallible code, and provide a set of cleanup instructions that will be
/// executed after the block. The cleanup can be jumped to early if there is a failure,
/// using the goto_cleanup_on_fail! macro.
/// ```
///enum Quality {
///    AfraidOfVacuum,
///    FindsSnacksInCatbox,
///}
///
/// pub struct Dog {
///    beauty: u128, // cannot be < 0
///    weight: u64,
///    personality: i64,
///    list_of_quirks: Vec<Quality>
///}
///
/// impl Dog {
///    fn internal_memory_thing(&mut self, i: &str) -> Result<(), ()> {
///        // take full ownership of quirks, leaving an empty vec in the struct field
///        // we have to remember to replace it, or the caller may find our Dog in
///        // an unexpected state.
///        let mut quirks = std::mem::replace(&mut self.list_of_quirks, Vec::new());
///
///        // the block is named (here, `'my_cleanup`) so that blocks can be nested
///        // within one another, and the proper block to break from can be specified.
///        with_cleanup!(('my_cleanup) {
///            // returns the value if Some, else jumps to the cleanup block.
///            let index = goto_cleanup_on_fail!(usize::parse(i), 'my_cleanup);
///
///        // the cleanup block is always preceded by "'cleanup:". This is not a variable,
///        // but rather marks the cleanup block.
///        } 'cleanup: {
///            // restore the quirks to their rightful field on the struct
///            self.list_of_quirks = quirks;
///        })
///    }
///}
///
///
/// ```
macro_rules! with_cleanup {
    (($label:tt) $code:block 'cleanup: $cleanup:block) => {
        $label: loop {
            $code

            break $label
        }

        $cleanup
    }
}

pub mod game;
pub mod interpreter;
pub mod item;
pub mod map;
pub mod player;
pub mod text;
