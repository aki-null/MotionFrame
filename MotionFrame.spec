# -*- mode: python ; coding: utf-8 -*-

a = Analysis(
    ['app/MotionFrame.py'],
    pathex=[],
    binaries=[],
    datas=[ ('app/translation', 'translation') ],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='MotionFrame',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
coll = COLLECT(
    exe,
    a.binaries,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='MotionFrame',
)
app = BUNDLE(
    coll,
    name='MotionFrame.app',
    icon='AppIcon.icns',
    bundle_identifier='com.akinull.MotionFrame',
    version='1.0.0',
)
