#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    
    // 设置应用程序图标（如果有的话）
    // res.set_icon("path/to/icon.ico");
    
    // 设置应用程序信息
    res.set("ProductName", "Auto Backup")
       .set("FileDescription", "Automatic Backup Tool")
       .set("LegalCopyright", "© 2024")
       .set("OriginalFilename", "auto_backup.exe");
    
    // Windows 兼容性清单
    res.set_manifest(r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <assemblyIdentity
        version="1.0.0.0"
        processorArchitecture="*"
        name="AutoBackup"
        type="win32"
    />
    <description>Automatic Backup Tool</description>
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
            </requestedPrivileges>
        </security>
    </trustInfo>
    <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
        <application>
            <!-- Windows 7 -->
            <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}"/>
            <!-- Windows 8 -->
            <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}"/>
            <!-- Windows 8.1 -->
            <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
            <!-- Windows 10/11 -->
            <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
        </application>
    </compatibility>
</assembly>
"#);

    res.compile().unwrap_or_else(|e| {
        eprintln!("Failed to compile resources: {}", e);
        std::process::exit(1);
    });
}

#[cfg(not(windows))]
fn main() {}
