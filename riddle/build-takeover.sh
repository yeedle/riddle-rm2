#!/bin/bash
# Build riddle in TAKEOVER mode (links libquill.so + vendor Qt/qsgepaper).
#
# Must link with the device SDK's gcc — the Ubuntu cross-linker's glibc can't
# resolve the GLIBC symbols the device's Qt libs require. The SDK gcc ships the
# matching sysroot.
#
# Device selection via $DEVICE:
#   DEVICE=rmpp (default) -> Paper Pro, aarch64, SDK ~/rm-sdk-3.26
#   DEVICE=rm2            -> reMarkable 2, armv7, SDK ~/rm-sdk-rm2
set -euo pipefail
cd "$(dirname "$0")"

DEVICE="${DEVICE:-rmpp}"

case "$DEVICE" in
  rm2)
    SDK="${SDK:-~/rm-sdk-rm2}"
    RUST_TARGET=armv7-unknown-linux-gnueabihf
    CARGO_LINKER_VAR=CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER
    CARGO_FEATURES="takeover,rm2"
    ;;
  rmpp)
    SDK="${SDK:-~/rm-sdk-3.26}"
    RUST_TARGET=aarch64-unknown-linux-gnu
    CARGO_LINKER_VAR=CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER
    CARGO_FEATURES="takeover"
    ;;
  *)
    echo "unknown DEVICE=$DEVICE (use rmpp or rm2)" >&2
    exit 1
    ;;
esac

# Expand a leading ~ in SDK.
SDK="${SDK/#\~/$HOME}"
ENV=$(ls "$SDK"/environment-setup-* | head -n1)
unset LD_LIBRARY_PATH          # SDK env refuses to source otherwise
source "$ENV"                  # sets CC=<triple>-gcc ... --sysroot=...

# Ensure quill's build artifacts exist (libquill.so + vendor/libqsgepaper.so).
if [ ! -f ../quill/build/libquill.so ]; then
    echo "building quill first..."
    ( cd ../quill && DEVICE="$DEVICE" SDK="$SDK" ./build.sh )
fi

# Point cargo's cross linker at the SDK gcc. $CC includes the -mcpu/-sysroot
# flags as one string; cargo wants a single program, so wrap it.
cat > /tmp/riddle-sdk-cc.sh <<EOF
#!/bin/bash
exec $CC "\$@"
EOF
chmod +x /tmp/riddle-sdk-cc.sh

export $CARGO_LINKER_VAR=/tmp/riddle-sdk-cc.sh
# rustc still targets the glibc triple; the SDK gcc just links.

cargo build --release --target "$RUST_TARGET" --features "$CARGO_FEATURES" "$@"

# The windowed (default-feature) build shares the same output path and would
# clobber this one. Copy the takeover binary to a distinct name so the two
# never collide.
OUT=target/$RUST_TARGET/release
cp "$OUT/riddle" "$OUT/riddle-takeover"
echo "built: $OUT/riddle-takeover ($DEVICE; takeover; libquill-linked)"
