use tailwind_fuse::merge::tw_merge;

pub fn cn(classes: impl AsRef<str>) -> String {
    tw_merge(classes.as_ref())
}
