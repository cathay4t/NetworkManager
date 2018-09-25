/* -*- Mode: C; tab-width: 4; indent-tabs-mode: t; c-basic-offset: 4 -*- */
/* NetworkManager system settings service
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * (C) Copyright 2010-2016 Red Hat, Inc.
 */

#ifndef __NMS_KEYFILE_UTILS_H__
#define __NMS_KEYFILE_UTILS_H__

#include "NetworkManagerUtils.h"

#define NMS_KEYFILE_PATH_NAME_LIB             NMLIBDIR "/profiles"
#define NMS_KEYFILE_PATH_NAME_ETC_DEFAULT     NMCONFDIR "/system-connections"
#define NMS_KEYFILE_PATH_NAME_RUN             NMRUNDIR "/profiles"

#define NMS_KEYFILE_PATH_SUFFIX_NMKEYFILE     ".nmkeyfile"
#define NMS_KEYFILE_PATH_PREFIX_NMLOADED      ".loaded-"

#define NMS_KEYFILE_ACTIVE_UUID_NULL          "/dev/null"

typedef enum {
	NMS_KEYFILE_FILETYPE_KEYFILE,
	NMS_KEYFILE_FILETYPE_NMLOADED,
} NMSKeyfileFiletype;

typedef enum {
	/* the order here matters. Higher numbers are more important. E.g. /etc shadows
	 * connections from /usr/lib. */
	NMS_KEYFILE_STORAGE_TYPE_LIB, /* from /usr/lib */
	NMS_KEYFILE_STORAGE_TYPE_ETC, /* from /etc */
	NMS_KEYFILE_STORAGE_TYPE_RUN, /* from /var/run */
	NMS_KEYFILE_STORAGE_TYPE_MEM, /* in-memory */
} NMSKeyfileStorageType;


void nms_keyfile_storage_type_get_info (NMSKeyfileStorageType storage_type,
                                        const char **out_dir_name);

gboolean nms_keyfile_storage_type_should_ignore_file (NMSKeyfileStorageType storage_type,
                                                      const char *filename);

/*****************************************************************************/

char *nms_keyfile_loaded_uuid_filename (const char *dirname,
                                        const char *uuid,
                                        gboolean temporary);

gboolean nms_keyfile_loaded_uuid_read (const char *dirname,
                                       const char *filename,
                                       char **out_full_filename,
                                       char **out_uuid,
                                       char **out_loaded_path);

gboolean nms_keyfile_loaded_uuid_read_from_file (const char *full_filename,
                                                 char **out_dirname,
                                                 char **out_filename,
                                                 char **out_uuid,
                                                 char **out_loaded_path);

gboolean nms_keyfile_loaded_uuid_write (const char *dirname,
                                        const char *uuid,
                                        const char *loaded_path,
                                        gboolean allow_relative,
                                        char **out_full_filename);

/*****************************************************************************/

struct stat;
gboolean nms_keyfile_utils_check_file_permissions_stat (NMSKeyfileFiletype filetype,
                                                        const struct stat *st,
                                                        GError **error);

gboolean nms_keyfile_utils_check_file_permissions (NMSKeyfileFiletype filetype,
                                                   const char *filename,
                                                   struct stat *out_st,
                                                   GError **error);

/*****************************************************************************/

char *nms_keyfile_utils_escape_filename (const char *filename);


#endif /* __NMS_KEYFILE_UTILS_H__ */
