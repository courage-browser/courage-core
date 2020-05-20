/* Copyright (c) 2020 The Brave Authors. All rights reserved.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef BAT_CONFIRMATIONS_INTERNAL_COUNTRY_SUBDIVISION_CODES_H_
#define BAT_CONFIRMATIONS_INTERNAL_COUNTRY_SUBDIVISION_CODES_H_

#include <map>
#include <set>
#include <string>

namespace ads {

using AllowedCountrySubdivisionsMap =
        std::map<std::string, std::set<std::string>>;

const AllowedCountrySubdivisionsMap kValidSubdivisionCountryCodes = {
  //  Format: { ISO Country Code, { ISO Country Subdivision codes ... } }
  {
    "US", {
        "US-AL",  // United States - Alabama
        "US-AK",  // United States - Alaska
        "US-AZ",  // United States - Arizona
        "US-AR",  // United States - Arkansas
        "US-CA",  // United States - California
        "US-CO",  // United States - Colorado
        "US-CT",  // United States - Connecticut
        "US-DE",  // United States - Delaware
        "US-FL",  // United States - Florida
        "US-GA",  // United States - Georgia
        "US-HI",  // United States - Hawaii
        "US-ID",  // United States - Idaho
        "US-IL",  // United States - Illinois
        "US-IN",  // United States - Indiana
        "US-IA",  // United States - Iowa
        "US-KS",  // United States - Kansas
        "US-KY",  // United States - Kentucky
        "US-LA",  // United States - Louisiana
        "US-ME",  // United States - Maine
        "US-MD",  // United States - Maryland
        "US-MA",  // United States - Massachusetts
        "US-MI",  // United States - Michigan
        "US-MN",  // United States - Minnesota
        "US-MS",  // United States - Mississippi
        "US-MO",  // United States - Missouri
        "US-MT",  // United States - Montana
        "US-NE",  // United States - Nebraska
        "US-NV",  // United States - Nevada
        "US-NH",  // United States - New Hampshire
        "US-NJ",  // United States - New Jersey
        "US-NM",  // United States - New Mexico
        "US-NY",  // United States - New York
        "US-NC",  // United States - North Carolina
        "US-ND",  // United States - North Dakota
        "US-OH",  // United States - Ohio
        "US-OK",  // United States - Oklahoma
        "US-OR",  // United States - Oregon
        "US-PA",  // United States - Pennsylvania
        "US-RI",  // United States - Rhode Island
        "US-SC",  // United States - South Carolina
        "US-SD",  // United States - South Dakota
        "US-TN",  // United States - Tennessee
        "US-TX",  // United States - Texas
        "US-UT",  // United States - Utah
        "US-VT",  // United States - Vermont
        "US-VA",  // United States - Virginia
        "US-WA",  // United States - Washington
        "US-WV",  // United States - West Virginia
        "US-WI",  // United States - Wisconsin
        "US-WY",  // United States - Wyoming
    }
  }
};

}  // namespace ads

#endif  // BAT_CONFIRMATIONS_INTERNAL_COUNTRY_SUBDIVISION_CODES_H_
