// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2025 Aalivexy
 
use std::{
    thread::{sleep, spawn},
    time::Duration,
};
use windows::{
    Security::Credentials::UI::{
        UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
    },
    Win32::{
        System::{
            Threading::{AttachThreadInput, GetCurrentThreadId},
            WinRT::IUserConsentVerifierInterop,
        },
        UI::{
            Input::KeyboardAndMouse::SetFocus,
            WindowsAndMessaging::{
                BringWindowToTop, FindWindowW, GetForegroundWindow, GetWindowThreadProcessId,
                HWND_DESKTOP, SetForegroundWindow,
            },
        },
    },
    core::{HSTRING, factory, w},
};
use windows_future::IAsyncOperation;

pub fn authenticate_with_biometrics() -> bool {
    spawn(|| {
        for _ in 0..40 {
            sleep(Duration::from_millis(50));
            center_security_prompt();
        }
    });
    unsafe {
        factory::<UserConsentVerifier, IUserConsentVerifierInterop>()
            .unwrap()
            .RequestVerificationForWindowAsync::<IAsyncOperation<UserConsentVerificationResult>>(
                HWND_DESKTOP,
                &HSTRING::new(),
            )
            .is_ok_and(|async_op| async_op.get() == Ok(UserConsentVerificationResult::Verified))
    }
}

pub fn get_biometrics_status() -> i32 {
    UserConsentVerifier::CheckAvailabilityAsync().map_or(5, |async_op| {
        async_op.get().map_or(5, |availability| {
            #[allow(non_snake_case)]
            match availability {
                UserConsentVerifierAvailability::Available => 0,
                UserConsentVerifierAvailability::DeviceNotPresent => 2,
                UserConsentVerifierAvailability::NotConfiguredForUser => 7,
                UserConsentVerifierAvailability::DisabledByPolicy => 5,
                UserConsentVerifierAvailability::DeviceBusy => 2,
                _ => 5,
            }
        })
    })
}

fn center_security_prompt() {
    let hwnd = unsafe { FindWindowW(w!("Credential Dialog Xaml Host"), None) };
    if let Ok(hwnd) = hwnd {
        unsafe {
            let fg_hwnd = GetForegroundWindow();
            let cur_id = GetCurrentThreadId();
            let fg_id = GetWindowThreadProcessId(fg_hwnd, None);
            let _ = AttachThreadInput(cur_id, fg_id, true);
            let _ = SetForegroundWindow(hwnd);
            let _ = BringWindowToTop(hwnd);
            let _ = SetFocus(Some(hwnd));
            let _ = AttachThreadInput(cur_id, fg_id, false);
        }
    }
}
