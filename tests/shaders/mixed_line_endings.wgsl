// LF, including a CRLF file after an LF one: each include line must end at its own line break.
@include shared/consts.wgsl
@include shared/crlf.wgsl

fn total() -> u32 {
    return VALUE + OTHER;
}
