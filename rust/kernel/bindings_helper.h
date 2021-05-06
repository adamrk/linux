/* SPDX-License-Identifier: GPL-2.0 */

#include <linux/cdev.h>
#include <linux/fs.h>
#include <linux/module.h>
#include <linux/proc_fs.h>
#include <linux/random.h>
#include <linux/seq_file.h>
#include <linux/slab.h>
#include <linux/sysctl.h>
#include <linux/uaccess.h>
#include <linux/version.h>
#include <linux/miscdevice.h>
#include <linux/poll.h>
#include <linux/mm.h>
#include <uapi/linux/android/binder.h>

#ifdef CONFIG_PROC_FS
#include "../../fs/proc/internal.h"
#endif

// `bindgen` gets confused at certain things
const gfp_t BINDINGS_GFP_KERNEL = GFP_KERNEL;
const gfp_t BINDINGS___GFP_ZERO = __GFP_ZERO;
