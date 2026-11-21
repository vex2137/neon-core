//% MAIN_ACTIVITY_BODY
    private static native void neonOnPause();

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
