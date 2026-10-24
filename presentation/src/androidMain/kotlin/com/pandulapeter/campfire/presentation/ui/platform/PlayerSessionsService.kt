/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.platform

import android.service.notification.NotificationListenerService

/**
 * What Android asks of an app that wants the media sessions of the other apps: a notification listener the user has
 * allowed, since a session is only handed to one. It listens to nothing - it overrides no callback, so no notification is
 * ever read - and exists to be the name `MediaSessionManager.getActiveSessions` is given; see [rememberExternalPlayerControl].
 */
class PlayerSessionsService : NotificationListenerService()
