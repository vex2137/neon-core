//% MAIN_ACTIVITY_BODY
    private static native void neonOnPause();
    private static native void neonOnGamepadAxis(int axisCode, float value);
    private static native void neonOnGamepadButton(int keyCode, boolean pressed);

    private static boolean isGamepadSource(int source) {
        return (source & android.view.InputDevice.SOURCE_GAMEPAD) == android.view.InputDevice.SOURCE_GAMEPAD
            || (source & android.view.InputDevice.SOURCE_JOYSTICK) == android.view.InputDevice.SOURCE_JOYSTICK;
    }

    private static float getCenteredAxis(android.view.MotionEvent event, android.view.InputDevice device, int axis) {
        if (device == null) {
            return event.getAxisValue(axis);
        }
        android.view.InputDevice.MotionRange range = device.getMotionRange(axis, event.getSource());
        if (range == null) {
            return 0.0f;
        }
        float value = event.getAxisValue(axis);
        return Math.abs(value) > range.getFlat() ? value : 0.0f;
    }

    @Override
    public boolean dispatchGenericMotionEvent(android.view.MotionEvent event) {
        if (event != null && isGamepadSource(event.getSource())
                && event.getAction() == android.view.MotionEvent.ACTION_MOVE) {
            android.view.InputDevice device = event.getDevice();
            float lx = getCenteredAxis(event, device, android.view.MotionEvent.AXIS_X);
            float ly = getCenteredAxis(event, device, android.view.MotionEvent.AXIS_Y);

            float rx = getCenteredAxis(event, device, android.view.MotionEvent.AXIS_Z);
            float ry = getCenteredAxis(event, device, android.view.MotionEvent.AXIS_RZ);
            float altRx = getCenteredAxis(event, device, android.view.MotionEvent.AXIS_RX);
            float altRy = getCenteredAxis(event, device, android.view.MotionEvent.AXIS_RY);
            if (Math.abs(altRx) > Math.abs(rx)) {
                rx = altRx;
            }
            if (Math.abs(altRy) > Math.abs(ry)) {
                ry = altRy;
            }

            float lt = Math.max(
                event.getAxisValue(android.view.MotionEvent.AXIS_LTRIGGER),
                event.getAxisValue(android.view.MotionEvent.AXIS_BRAKE)
            );
            float rt = Math.max(
                event.getAxisValue(android.view.MotionEvent.AXIS_RTRIGGER),
                event.getAxisValue(android.view.MotionEvent.AXIS_GAS)
            );
            float hatX = event.getAxisValue(android.view.MotionEvent.AXIS_HAT_X);
            float hatY = event.getAxisValue(android.view.MotionEvent.AXIS_HAT_Y);

            try {
                neonOnGamepadAxis(0, lx);
                neonOnGamepadAxis(1, ly);
                neonOnGamepadAxis(2, rx);
                neonOnGamepadAxis(3, ry);
                neonOnGamepadAxis(4, lt);
                neonOnGamepadAxis(5, rt);
                neonOnGamepadAxis(6, hatX);
                neonOnGamepadAxis(7, hatY);
            } catch (Throwable ignored) {
            }
            return true;
        }
        return super.dispatchGenericMotionEvent(event);
    }

    private static int mapGamepadKeyCode(int keyCode) {
        switch (keyCode) {
            case android.view.KeyEvent.KEYCODE_BUTTON_A: return 0;
            case android.view.KeyEvent.KEYCODE_BUTTON_B: return 1;
            case android.view.KeyEvent.KEYCODE_BUTTON_X: return 2;
            case android.view.KeyEvent.KEYCODE_BUTTON_Y: return 3;
            case android.view.KeyEvent.KEYCODE_BUTTON_L1: return 4;
            case android.view.KeyEvent.KEYCODE_BUTTON_R1: return 5;
            case android.view.KeyEvent.KEYCODE_BUTTON_L2: return 6;
            case android.view.KeyEvent.KEYCODE_BUTTON_R2: return 7;
            case android.view.KeyEvent.KEYCODE_BUTTON_START: return 8;
            case android.view.KeyEvent.KEYCODE_BUTTON_SELECT: return 9;
            case android.view.KeyEvent.KEYCODE_DPAD_UP: return 10;
            case android.view.KeyEvent.KEYCODE_DPAD_DOWN: return 11;
            case android.view.KeyEvent.KEYCODE_DPAD_LEFT: return 12;
            case android.view.KeyEvent.KEYCODE_DPAD_RIGHT: return 13;
            default: return -1;
        }
    }

    @Override
    public boolean dispatchKeyEvent(android.view.KeyEvent event) {
        if (event != null) {
            int source = event.getSource();
            boolean fromController = isGamepadSource(source)
                || (source & android.view.InputDevice.SOURCE_DPAD) == android.view.InputDevice.SOURCE_DPAD;
            int mapped = mapGamepadKeyCode(event.getKeyCode());
            if (mapped >= 0 && (fromController || mapped <= 9)) {
                int action = event.getAction();
                if (action == android.view.KeyEvent.ACTION_DOWN) {
                    if (event.getRepeatCount() == 0) {
                        try {
                            neonOnGamepadButton(mapped, true);
                        } catch (Throwable ignored) {
                        }
                    }
                    return true;
                } else if (action == android.view.KeyEvent.ACTION_UP) {
                    try {
                        neonOnGamepadButton(mapped, false);
                    } catch (Throwable ignored) {
                    }
                    return true;
                }
            }
        }
        return super.dispatchKeyEvent(event);
    }

    private void applyImmersiveFullscreen() {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                android.view.Window window = getWindow();
                if (window == null) {
                    return;
                }
                window.addFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
                window.addFlags(android.view.WindowManager.LayoutParams.FLAG_FULLSCREEN);
                if (android.os.Build.VERSION.SDK_INT >= 28) {
                    android.view.WindowManager.LayoutParams lp = window.getAttributes();
                    lp.layoutInDisplayCutoutMode =
                        android.view.WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES;
                    window.setAttributes(lp);
                }
                if (android.os.Build.VERSION.SDK_INT >= 30) {
                    window.setDecorFitsSystemWindows(false);
                    android.view.WindowInsetsController controller = window.getInsetsController();
                    if (controller != null) {
                        controller.hide(
                            android.view.WindowInsets.Type.statusBars()
                                | android.view.WindowInsets.Type.navigationBars()
                        );
                        controller.setSystemBarsBehavior(
                            android.view.WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
                        );
                    }
                } else {
                    android.view.View decorView = window.getDecorView();
                    if (decorView != null) {
                        int uiOptions = android.view.View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                            | android.view.View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                            | android.view.View.SYSTEM_UI_FLAG_FULLSCREEN
                            | android.view.View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                            | android.view.View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                            | android.view.View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN;
                        decorView.setSystemUiVisibility(uiOptions);
                    }
                }
            }
        });
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            applyImmersiveFullscreen();
        } else {
            try {
                neonOnPause();
            } catch (Throwable ignored) {
            }
        }
    }
//% END

//% MAIN_ACTIVITY_ON_CREATE
        applyImmersiveFullscreen();
//% END

//% MAIN_ACTIVITY_ON_RESUME
        applyImmersiveFullscreen();
//% END

//% MAIN_ACTIVITY_ON_PAUSE
        try {
            neonOnPause();
        } catch (Throwable ignored) {
        }
//% END
