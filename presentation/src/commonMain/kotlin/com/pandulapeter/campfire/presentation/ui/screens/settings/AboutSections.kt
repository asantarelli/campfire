/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.screens.settings

import androidx.compose.runtime.Composable
import com.pandulapeter.campfire.presentation.CAMPFIRE_VERSION_NAME
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.ic_sd_mark
import com.pandulapeter.campfire.presentation.resources.ic_git_hub
import com.pandulapeter.campfire.presentation.resources.ic_license
import com.pandulapeter.campfire.presentation.resources.ic_star
import com.pandulapeter.campfire.presentation.resources.settings_created_by
import com.pandulapeter.campfire.presentation.resources.settings_distribution_app_store
import com.pandulapeter.campfire.presentation.resources.settings_distribution_mac_app_store
import com.pandulapeter.campfire.presentation.resources.settings_distribution_microsoft_store
import com.pandulapeter.campfire.presentation.resources.settings_distribution_play_store
import com.pandulapeter.campfire.presentation.resources.settings_git_hub
import com.pandulapeter.campfire.presentation.resources.settings_git_hub_description
import com.pandulapeter.campfire.presentation.resources.settings_group_campfire
import com.pandulapeter.campfire.presentation.resources.settings_group_help
import com.pandulapeter.campfire.presentation.resources.settings_group_legal
import com.pandulapeter.campfire.presentation.resources.settings_open_source_licenses
import com.pandulapeter.campfire.presentation.resources.settings_open_source_licenses_description
import com.pandulapeter.campfire.presentation.resources.settings_rate
import com.pandulapeter.campfire.presentation.resources.settings_rate_description
import com.pandulapeter.campfire.presentation.resources.settings_version
import com.pandulapeter.campfire.presentation.ui.CampfireViewModel
import com.pandulapeter.campfire.presentation.ui.screens.settings.LinkListItem
import com.pandulapeter.campfire.presentation.ui.dialogs.DialogType
import com.pandulapeter.campfire.presentation.ui.platform.Distribution
import com.pandulapeter.campfire.presentation.ui.platform.platformStore
import com.pandulapeter.campfire.presentation.ui.settingsPage.SettingsSection
import org.jetbrains.compose.resources.painterResource

/**
 * What the app is and where it lives, in three groups: the app itself (how to give something back, and, last, whose
 * work it is built on and which version this is); help (the source code and issue tracker of this fork); and the fine
 * print. The row that credits the work leads to the original Campfire project, since the license asks for it to stay
 * named and that is where its author is found. There is no website, privacy policy page or donation link: the app
 * is not published anywhere but through the source code, which is also what GitHub is to somebody who has never used it.
 *
 * The rating row leads to the store of the platform the app is running on, and only once that listing exists. It
 * says "rate", never "get it from the store": a store page has an install button on it, and somebody who has the
 * direct download would end up with a second copy of the app, sandboxed, with a library of its own.
 */
@Composable
internal fun AboutSections(
    viewModel: CampfireViewModel,
    urlOpener: (String) -> Unit,
) {
    AppSection(urlOpener = urlOpener)
    HelpSection(urlOpener = urlOpener)
    LegalSection(urlOpener = urlOpener, onOpenSourceLicensesClicked = { viewModel.dialogHost.showDialog(DialogType.OpenSourceLicenses) })
}

@Composable
private fun AppSection(
    urlOpener: (String) -> Unit,
) = SettingsSection(title = stringResource(Res.string.settings_group_campfire)) {
    platformStore?.let { store ->
        store.listingUrl?.let { listingUrl ->
            LinkListItem(
                title = stringResource(Res.string.settings_rate),
                description = stringResource(Res.string.settings_rate_description, stringResource(store.storeName)),
                icon = painterResource(Res.drawable.ic_star),
                onClick = { urlOpener(listingUrl) },
            )
        }
    }
    LinkListItem(
        title = stringResource(Res.string.settings_created_by),
        description = stringResource(Res.string.settings_version, CAMPFIRE_VERSION_NAME),
        icon = painterResource(Res.drawable.ic_sd_mark),
        onClick = { urlOpener(CAMPFIRE_URL) },
    )
}

@Composable
private fun HelpSection(
    urlOpener: (String) -> Unit,
) = SettingsSection(title = stringResource(Res.string.settings_group_help)) {
    LinkListItem(
        title = stringResource(Res.string.settings_git_hub),
        description = stringResource(Res.string.settings_git_hub_description),
        icon = painterResource(Res.drawable.ic_git_hub),
        onClick = { urlOpener(GIT_HUB_URL) },
    )
}

@Composable
private fun LegalSection(
    urlOpener: (String) -> Unit,
    onOpenSourceLicensesClicked: () -> Unit,
) = SettingsSection(title = stringResource(Res.string.settings_group_legal)) {
    LinkListItem(
        title = stringResource(Res.string.settings_open_source_licenses),
        description = stringResource(Res.string.settings_open_source_licenses_description),
        icon = painterResource(Res.drawable.ic_license),
        onClick = onOpenSourceLicensesClicked,
    )
}

/** The name a rating is left under, which is the one thing the app still has to call a store. */
private val Distribution.storeName
    get() = when (this) {
        Distribution.PLAY_STORE -> Res.string.settings_distribution_play_store
        Distribution.APP_STORE -> Res.string.settings_distribution_app_store
        Distribution.MAC_APP_STORE -> Res.string.settings_distribution_mac_app_store
        Distribution.MICROSOFT_STORE -> Res.string.settings_distribution_microsoft_store
    }

private const val GIT_HUB_URL = "https://github.com/asantarelli/campfire"
private const val CAMPFIRE_URL = "https://github.com/pandulapeter/campfire"
