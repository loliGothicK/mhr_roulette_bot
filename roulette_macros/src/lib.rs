// ISC License
//
// Copyright (c) 2021 Mitama Lab
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

mod source_info {
    #[macro_export]
    macro_rules! func_sig {
        () => {{
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);
            &name[..name.len() - 3]
        }};
    }

    #[macro_export]
    macro_rules! pretty_info {
        () => {
            format!(
                "{} (in {} [{}:{}:{}])",
                $crate::func_sig!(),
                module_path!(),
                file!(),
                line!(),
                column!()
            )
        };
    }

    #[macro_export]
    macro_rules! bailout {
        ($context:literal, $err:expr) => {
            return Err(anyhow::Error::from($err).context($context));
        };
    }
}
