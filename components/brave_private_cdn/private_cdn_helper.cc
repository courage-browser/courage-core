/* Copyright (c) 2020 The Brave Authors. All rights reserved.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "brave/components/brave_private_cdn/private_cdn_helper.h"

#include "base/big_endian.h"

namespace {

bool GetPayloadExtent(
    base::StringPiece padded_string,
    size_t* start,
    size_t* length) {
  DCHECK(start);
  DCHECK(length);

  if (padded_string.size() < sizeof(uint32_t)) {
    return false;  // Missing length field
  }

  // Read payload length from the header.
  uint32_t data_length;
  base::ReadBigEndian(padded_string.data(), &data_length);
  if (padded_string.size() < data_length + sizeof(uint32_t)) {
    return false;  // Payload shorter than expected length
  }

  *start = sizeof(uint32_t);
  *length = data_length;
  return true;
}

}  // namespace

namespace brave {

bool PrivateCdnHelper::RemovePadding(std::string* padded_string) const {
  if (!padded_string) {
    return false;
  }

  size_t start;
  size_t length;
  if (!GetPayloadExtent(*padded_string, &start, &length)) {
    return false;
  }

  padded_string->erase(0, start);
  padded_string->resize(length);
  return true;
}

bool PrivateCdnHelper::RemovePadding(base::StringPiece* padded_string) const {
  if (!padded_string) {
    return false;
  }

  size_t start;
  size_t length;
  if (!GetPayloadExtent(*padded_string, &start, &length)) {
    return false;
  }

  padded_string->remove_prefix(start);
  padded_string->remove_suffix(padded_string->size() - length);
  return true;
}

PrivateCdnHelper::PrivateCdnHelper() = default;

PrivateCdnHelper::~PrivateCdnHelper() = default;

}  // namespace brave
