# BizClaw ProGuard Rules
# ================================

# Keep Kotlin serialization
-keepattributes *Annotation*, InnerClasses
-dontnote kotlinx.serialization.AnnotationsKt

-keepclassmembers class kotlinx.serialization.json.** {
    *** Companion;
}
-keepclasseswithmembers class kotlinx.serialization.json.** {
    kotlinx.serialization.KSerializer serializer(...);
}

-keep,includedescriptorclasses class vn.bizclaw.app.**$$serializer { *; }
-keepclassmembers class vn.bizclaw.app.** {
    *** Companion;
}
-keepclasseswithmembers class vn.bizclaw.app.** {
    kotlinx.serialization.KSerializer serializer(...);
}

# Keep Retrofit
-keepattributes Signature, Exceptions
-keep class retrofit2.** { *; }
-keepclasseswithmembers class * {
    @retrofit2.http.* <methods>;
}

# Keep OkHttp
-dontwarn okhttp3.**
-dontwarn okio.**

# Keep Compose
-keep class androidx.compose.** { *; }

# Keep BizClaw native FFI (when bizclaw.so is loaded)
-keep class vn.bizclaw.app.native.** { *; }
