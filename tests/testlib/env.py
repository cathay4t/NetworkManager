# SPDX-License-Identifier: Apache-2.0

import os

def is_fedora():
    return os.path.exists("/etc/fedora-release")
