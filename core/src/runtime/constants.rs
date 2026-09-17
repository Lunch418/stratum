//! Константы языка: PI, WM_XXX, стили окон, флаги графики.
//!
//! Файл сгенерирован `tools/gen_constants.py` из `docs/lang/constants.json`
//! (исходник — `template/CONSTANT.TPL` установленного Stratum). Не править вручную.

/// Имя в нижнем регистре и значение; отсортировано для двоичного поиска.
pub const CONSTANTS: [(&str, f64); 414] = [
    ("attrput", 3.0), // Tool types. By GetToolType
    ("attrreset", 2.0), // Tool types. By GetToolType
    ("attrset", 1.0), // Tool types. By GetToolType
    ("bbt_oriented_common", 1.0), // Ogre Wrapper
    ("bbt_oriented_self", 2.0), // Ogre Wrapper
    ("bbt_perpendicular_common", 3.0), // Ogre Wrapper
    ("bbt_perpendicular_self", 4.0), // Ogre Wrapper
    ("bbt_point", 0.0), // Ogre Wrapper
    ("brush2d", 2.0), // Инструменты
    ("bs_gradient_linear", 10.0), // Стили кисти
    ("bs_hatched", 2.0), // Стили кисти
    ("bs_null", 1.0), // Стили кисти
    ("bs_pattern", 3.0), // Стили кисти
    ("bs_solid", 0.0), // Стили кисти
    ("cmpf_always_fail", 0.0), // Ogre Wrapper
    ("cmpf_always_pass", 1.0), // Ogre Wrapper
    ("cmpf_equal", 4.0), // Ogre Wrapper
    ("cmpf_greater", 7.0), // Ogre Wrapper
    ("cmpf_greater_equal", 6.0), // Ogre Wrapper
    ("cmpf_less", 2.0), // Ogre Wrapper
    ("cmpf_less_equal", 3.0), // Ogre Wrapper
    ("cmpf_not_equal", 5.0), // Ogre Wrapper
    ("cull_anticlockwise", 3.0), // Ogre Wrapper
    ("cull_clockwise", 2.0), // Ogre Wrapper
    ("cull_none", 1.0), // Ogre Wrapper
    ("cur_arrow", 0.0), // Стандартные курсоры мыши
    ("cur_cross", 1.0), // Стандартные курсоры мыши
    ("cur_help", 12.0), // Стандартные курсоры мыши
    ("cur_link", 11.0), // Стандартные курсоры мыши
    ("cur_no", 3.0), // Стандартные курсоры мыши
    ("cur_size", 4.0), // Стандартные курсоры мыши
    ("cur_sizenesw", 5.0), // Стандартные курсоры мыши
    ("cur_sizens", 6.0), // Стандартные курсоры мыши
    ("cur_sizenwse", 7.0), // Стандартные курсоры мыши
    ("cur_sizewe", 8.0), // Стандартные курсоры мыши
    ("cur_text", 2.0), // Стандартные курсоры мыши
    ("cur_uparrow", 9.0), // Стандартные курсоры мыши
    ("cur_wait", 10.0), // Стандартные курсоры мыши
    ("dib2d", 3.0), // Инструменты
    ("doubledib2d", 4.0), // Инструменты
    ("file_attribute_archive", 32.0), // флаги для функции GetFileList
    ("file_attribute_compressed", 2048.0), // флаги для функции GetFileList
    ("file_attribute_directory", 16.0), // флаги для функции GetFileList
    ("file_attribute_hidden", 2.0), // флаги для функции GetFileList
    ("file_attribute_normal", 128.0), // флаги для функции GetFileList
    ("file_attribute_offline", 4096.0), // флаги для функции GetFileList
    ("file_attribute_readonly", 1.0), // флаги для функции GetFileList
    ("file_attribute_system", 4.0), // флаги для функции GetFileList
    ("file_attribute_temporary", 256.0), // флаги для функции GetFileList
    ("flddbbinary", 521.0), // Database constants (Field types). by AK
    ("flddbbool", 516.0), // Database constants (Field types). by AK
    ("flddbbytes", 522.0), // Database constants (Field types). by AK
    ("flddbchar", 513.0), // Database constants (Field types). by AK
    ("flddbdate", 517.0), // Database constants (Field types). by AK
    ("flddbfloat", 518.0), // Database constants (Field types). by AK
    ("flddblock", 519.0), // Database constants (Field types). by AK
    ("flddbmemo", 515.0), // Database constants (Field types). by AK
    ("flddbnum", 514.0), // Database constants (Field types). by AK
    ("flddboleblob", 520.0), // Database constants (Field types). by AK
    ("float32", 6.0), // Streams
    ("float40", 7.0), // Streams
    ("float64", 8.0), // Streams
    ("float80", 9.0), // Streams
    ("fog_exp", 1.0), // Ogre Wrapper
    ("fog_exp2", 2.0), // Ogre Wrapper
    ("fog_linear", 3.0), // Ogre Wrapper
    ("fog_none", 0.0), // Ogre Wrapper
    ("font2d", 5.0), // Инструменты
    ("gha_center", 1.0), // Ogre Wrapper
    ("gha_left", 0.0), // Ogre Wrapper
    ("gha_right", 2.0), // Ogre Wrapper
    ("gmm_pixels", 1.0), // Ogre Wrapper
    ("gmm_relative", 0.0), // Ogre Wrapper
    ("gmm_relative_aspect_adjusted", 2.0), // Ogre Wrapper
    ("gva_bottom", 2.0), // Ogre Wrapper
    ("gva_center", 1.0), // Ogre Wrapper
    ("gva_top", 0.0), // Ogre Wrapper
    ("hs_bdiagonal", 3.0), // Стили штриховки
    ("hs_cross", 4.0), // Стили штриховки
    ("hs_diagcross", 5.0), // Стили штриховки
    ("hs_fdiagonal", 2.0), // Стили штриховки
    ("hs_horizontal", 0.0), // Стили штриховки
    ("hs_vertical", 1.0), // Стили штриховки
    ("idx_caseinsens", 128.0), // Database constants (index). by AK
    ("idx_descending", 4.0), // Database constants (index). by AK
    ("idx_expr", 32.0), // Database constants (index). by AK
    ("idx_idxood", 64.0), // Database constants (index). by AK
    ("idx_maintained", 8.0), // Database constants (index). by AK
    ("idx_primary", 1.0), // Database constants (index). by AK
    ("idx_subset", 16.0), // Database constants (index). by AK
    ("idx_unique", 2.0), // Database constants (index). by AK
    ("int16", 3.0), // Streams
    ("int32", 5.0), // Streams
    ("int8", 1.0), // Streams
    ("lt_directional", 1.0), // Ogre Wrapper
    ("lt_point", 0.0), // Ogre Wrapper
    ("lt_spotlight", 2.0), // Ogre Wrapper
    ("mb_abortretryignore", 2.0), // MESSAGEBOX constants
    ("mb_applmodal", 0.0), // MESSAGEBOX constants
    ("mb_defbutton1", 0.0), // MESSAGEBOX constants
    ("mb_defbutton2", 256.0), // MESSAGEBOX constants
    ("mb_defbutton3", 512.0), // MESSAGEBOX constants
    ("mb_defmask", 3840.0), // MESSAGEBOX constants
    ("mb_iconasterisk", 64.0), // MESSAGEBOX constants
    ("mb_iconexclamation", 48.0), // MESSAGEBOX constants
    ("mb_iconhand", 16.0), // MESSAGEBOX constants
    ("mb_iconinformation", 64.0), // MESSAGEBOX constants
    ("mb_iconmask", 240.0), // MESSAGEBOX constants
    ("mb_iconquestion", 32.0), // MESSAGEBOX constants
    ("mb_iconstop", 16.0), // MESSAGEBOX constants
    ("mb_nofocus", 32768.0), // MESSAGEBOX constants
    ("mb_ok", 0.0), // MESSAGEBOX constants
    ("mb_okcancel", 1.0), // MESSAGEBOX constants
    ("mb_retrycancel", 5.0), // MESSAGEBOX constants
    ("mb_systemmodal", 4096.0), // MESSAGEBOX constants
    ("mb_taskmodal", 8192.0), // MESSAGEBOX constants
    ("mb_typemask", 15.0), // MESSAGEBOX constants
    ("mb_yesno", 4.0), // MESSAGEBOX constants
    ("mb_yesnocancel", 3.0), // MESSAGEBOX constants
    ("mcierr_bad_constant", 290.0), // MCI Error codes
    ("mcierr_bad_integer", 270.0), // MCI Error codes
    ("mcierr_bad_time_format", 293.0), // MCI Error codes
    ("mcierr_cannot_load_driver", 266.0), // MCI Error codes
    ("mcierr_cannot_use_all", 279.0), // MCI Error codes
    ("mcierr_createwindow", 347.0), // MCI Error codes
    ("mcierr_device_length", 310.0), // MCI Error codes
    ("mcierr_device_locked", 288.0), // MCI Error codes
    ("mcierr_device_not_installed", 306.0), // MCI Error codes
    ("mcierr_device_not_ready", 276.0), // MCI Error codes
    ("mcierr_device_open", 265.0), // MCI Error codes
    ("mcierr_device_ord_length", 311.0), // MCI Error codes
    ("mcierr_device_type_required", 287.0), // MCI Error codes
    ("mcierr_driver", 278.0), // MCI Error codes
    ("mcierr_driver_internal", 272.0), // MCI Error codes
    ("mcierr_duplicate_alias", 289.0), // MCI Error codes
    ("mcierr_duplicate_flags", 295.0), // MCI Error codes
    ("mcierr_extension_not_found", 281.0), // MCI Error codes
    ("mcierr_extra_characters", 305.0), // MCI Error codes
    ("mcierr_file_not_found", 275.0), // MCI Error codes
    ("mcierr_file_not_saved", 286.0), // MCI Error codes
    ("mcierr_file_read", 348.0), // MCI Error codes
    ("mcierr_file_write", 349.0), // MCI Error codes
    ("mcierr_filename_required", 304.0), // MCI Error codes
    ("mcierr_flags_not_compatible", 284.0), // MCI Error codes
    ("mcierr_get_cd", 307.0), // MCI Error codes
    ("mcierr_hardware", 262.0), // MCI Error codes
    ("mcierr_illegal_for_auto_open", 303.0), // MCI Error codes
    ("mcierr_internal", 277.0), // MCI Error codes
    ("mcierr_invalid_device_id", 257.0), // MCI Error codes
    ("mcierr_invalid_device_name", 263.0), // MCI Error codes
    ("mcierr_invalid_file", 296.0), // MCI Error codes
    ("mcierr_missing_command_string", 267.0), // MCI Error codes
    ("mcierr_missing_device_name", 292.0), // MCI Error codes
    ("mcierr_missing_parameter", 273.0), // MCI Error codes
    ("mcierr_missing_string_argument", 269.0), // MCI Error codes
    ("mcierr_multiple", 280.0), // MCI Error codes
    ("mcierr_must_use_shareable", 291.0), // MCI Error codes
    ("mcierr_new_requires_alias", 299.0), // MCI Error codes
    ("mcierr_no_closing_quote", 294.0), // MCI Error codes
    ("mcierr_no_element_allowed", 301.0), // MCI Error codes
    ("mcierr_no_integer", 312.0), // MCI Error codes
    ("mcierr_no_window", 346.0), // MCI Error codes
    ("mcierr_nonapplicable_function", 302.0), // MCI Error codes
    ("mcierr_notify_on_auto_open", 300.0), // MCI Error codes
    ("mcierr_null_parameter_block", 297.0), // MCI Error codes
    ("mcierr_out_of_memory", 264.0), // MCI Error codes
    ("mcierr_outofrange", 282.0), // MCI Error codes
    ("mcierr_param_overflow", 268.0), // MCI Error codes
    ("mcierr_parser_internal", 271.0), // MCI Error codes
    ("mcierr_seq_div_incompatible", 336.0), // MCI Error codes
    ("mcierr_seq_nomidipresent", 343.0), // MCI Error codes
    ("mcierr_seq_port_inuse", 337.0), // MCI Error codes
    ("mcierr_seq_port_mapnodevice", 339.0), // MCI Error codes
    ("mcierr_seq_port_miscerror", 340.0), // MCI Error codes
    ("mcierr_seq_port_nonexistent", 338.0), // MCI Error codes
    ("mcierr_seq_portunspecified", 342.0), // MCI Error codes
    ("mcierr_seq_timer", 341.0), // MCI Error codes
    ("mcierr_set_cd", 308.0), // MCI Error codes
    ("mcierr_set_drive", 309.0), // MCI Error codes
    ("mcierr_unnamed_resource", 298.0), // MCI Error codes
    ("mcierr_unrecognized_command", 261.0), // MCI Error codes
    ("mcierr_unrecognized_keyword", 259.0), // MCI Error codes
    ("mcierr_unsupported_function", 274.0), // MCI Error codes
    ("mcierr_wave_inputsinuse", 322.0), // MCI Error codes
    ("mcierr_wave_inputsunsuitable", 328.0), // MCI Error codes
    ("mcierr_wave_inputunspecified", 325.0), // MCI Error codes
    ("mcierr_wave_outputsinuse", 320.0), // MCI Error codes
    ("mcierr_wave_outputsunsuitable", 326.0), // MCI Error codes
    ("mcierr_wave_outputunspecified", 324.0), // MCI Error codes
    ("mcierr_wave_setinputinuse", 323.0), // MCI Error codes
    ("mcierr_wave_setinputunsuitable", 329.0), // MCI Error codes
    ("mcierr_wave_setoutputinuse", 321.0), // MCI Error codes
    ("mcierr_wave_setoutputunsuitable", 327.0), // MCI Error codes
    ("mtha_center", 1.0), // Ogre Wrapper
    ("mtha_left", 0.0), // Ogre Wrapper
    ("mtva_above", 1.0), // Ogre Wrapper
    ("mtva_below", 0.0), // Ogre Wrapper
    ("mtva_center", 2.0), // Ogre Wrapper
    ("nui_sp_ankle_left", 14.0), // NUI
    ("nui_sp_ankle_right", 18.0), // NUI
    ("nui_sp_count", 20.0), // NUI
    ("nui_sp_elbow_left", 5.0), // NUI
    ("nui_sp_elbow_right", 9.0), // NUI
    ("nui_sp_foot_left", 15.0), // NUI
    ("nui_sp_foot_right", 19.0), // NUI
    ("nui_sp_hand_left", 7.0), // NUI
    ("nui_sp_hand_right", 11.0), // NUI
    ("nui_sp_head", 3.0), // NUI
    ("nui_sp_hip_center", 0.0), // NUI
    ("nui_sp_hip_left", 12.0), // NUI
    ("nui_sp_hip_right", 16.0), // NUI
    ("nui_sp_knee_left", 13.0), // NUI
    ("nui_sp_knee_right", 17.0), // NUI
    ("nui_sp_shoulder_center", 2.0), // NUI
    ("nui_sp_shoulder_left", 4.0), // NUI
    ("nui_sp_shoulder_right", 8.0), // NUI
    ("nui_sp_spine", 1.0), // NUI
    ("nui_sp_wrist_left", 6.0), // NUI
    ("nui_sp_wrist_right", 10.0), // NUI
    ("oid_axis3d", 32100.0), // ------------- 3d -----------------------
    ("oid_frame2d", 32010.0), // ------------- resereved objects HANDLE`s -----------
    ("oid_frame2d1", 32002.0), // ------------- resereved objects HANDLE`s -----------
    ("oid_frame2d2", 32003.0), // ------------- resereved objects HANDLE`s -----------
    ("oid_frame2d3", 32004.0), // ------------- resereved objects HANDLE`s -----------
    ("oid_frame2d4", 32005.0), // ------------- resereved objects HANDLE`s -----------
    ("oid_frame2d5", 32006.0), // ------------- resereved objects HANDLE`s -----------
    ("oid_frame2d6", 32007.0), // ------------- resereved objects HANDLE`s -----------
    ("oid_frame2d7", 32008.0), // ------------- resereved objects HANDLE`s -----------
    ("oid_frame2d8", 32009.0), // ------------- resereved objects HANDLE`s -----------
    ("oid_frame3d", 32110.0), // ------------- 3d -----------------------
    ("oid_frame3d1", 32102.0), // ------------- 3d -----------------------
    ("oid_frame3d2", 32103.0), // ------------- 3d -----------------------
    ("oid_frame3d3", 32104.0), // ------------- 3d -----------------------
    ("oid_frame3d4", 32105.0), // ------------- 3d -----------------------
    ("oid_frame3d5", 32106.0), // ------------- 3d -----------------------
    ("oid_frame3d6", 32107.0), // ------------- 3d -----------------------
    ("oid_frame3d7", 32108.0), // ------------- 3d -----------------------
    ("oid_frame3d8", 32109.0), // ------------- 3d -----------------------
    ("oid_rcenter", 32001.0), // ------------- resereved objects HANDLE`s -----------
    ("oid_reserved", 32000.0), // ------------- resereved objects HANDLE`s -----------
    ("ot_line_list", 2.0), // Ogre Wrapper
    ("ot_line_strip", 3.0), // Ogre Wrapper
    ("ot_point_list", 1.0), // Ogre Wrapper
    ("ot_triangle_fan", 6.0), // Ogre Wrapper
    ("ot_triangle_list", 4.0), // Ogre Wrapper
    ("ot_triangle_strip", 5.0), // Ogre Wrapper
    ("otaxis3d", 53.0), // Windows messages
    ("otbitmap_2d", 21.0), // Windows messages
    ("otbrush2d", 102.0), // Tool types. By GetToolType
    ("otdib2d", 103.0), // Tool types. By GetToolType
    ("otdoublebitmap_2d", 22.0), // Windows messages
    ("otdoubledib2d", 104.0), // Tool types. By GetToolType
    ("oteditframe", 50.0), // Windows messages
    ("otfont2d", 105.0), // Tool types. By GetToolType
    ("otframe3d", 52.0), // Windows messages
    ("otgroup2d", 3.0), // Windows messages
    ("otgroup3d", 5.0), // Windows messages
    ("otline_2d", 20.0), // Windows messages
    ("otobject3d", 10.0), // Windows messages
    ("otpen2d", 101.0), // Tool types. By GetToolType
    ("otreftodib2d", 110.0), // Tool types. By GetToolType
    ("otreftodoubledib2d", 111.0), // Tool types. By GetToolType
    ("otrgroup2d", 4.0), // Windows messages
    ("otrotatecenter", 51.0), // Windows messages
    ("otstring2d", 106.0), // Tool types. By GetToolType
    ("ottext2d", 107.0), // Tool types. By GetToolType
    ("ottext_2d", 23.0), // Windows messages
    ("otview3d_2d", 24.0), // Windows messages
    ("pen2d", 1.0), // Инструменты
    ("pfc_3d", 128.0), // Paste from Clipboard
    ("pfc_all", 255.0), // Paste from Clipboard
    ("pfc_bruhs", 2.0), // Paste from Clipboard
    ("pfc_ddibs", 8.0), // Paste from Clipboard
    ("pfc_dibs", 4.0), // Paste from Clipboard
    ("pfc_fonts", 32.0), // Paste from Clipboard
    ("pfc_moveobject", 32768.0), // Paste from Clipboard
    ("pfc_pens", 1.0), // Paste from Clipboard
    ("pfc_setcurrent", 16384.0), // Paste from Clipboard
    ("pfc_setframe", 8192.0), // Paste from Clipboard
    ("pfc_strings", 16.0), // Paste from Clipboard
    ("pfc_texts", 64.0), // Paste from Clipboard
    ("pi", 3.1415926536), // PI
    ("pm_points", 1.0), // Ogre Wrapper
    ("pm_solid", 3.0), // Ogre Wrapper
    ("pm_wireframe", 2.0), // Ogre Wrapper
    ("ps_dash", 1.0), // Типы линий
    ("ps_dashdot", 3.0), // Типы линий
    ("ps_dashdotdot", 4.0), // Типы линий
    ("ps_dot", 2.0), // Типы линий
    ("ps_insideframe", 6.0), // Типы линий
    ("ps_null", 5.0), // Типы линий
    ("ps_solid", 0.0), // Типы линий
    ("pt_bool", 0.0), // Ogre Wrapper
    ("pt_colourvalue", 13.0), // Ogre Wrapper
    ("pt_int", 2.0), // Ogre Wrapper
    ("pt_long", 6.0), // Ogre Wrapper
    ("pt_matrix3", 10.0), // Ogre Wrapper
    ("pt_matrix4", 11.0), // Ogre Wrapper
    ("pt_orthographic", 0.0), // Ogre Wrapper
    ("pt_perspective", 1.0), // Ogre Wrapper
    ("pt_quaternion", 12.0), // Ogre Wrapper
    ("pt_real", 1.0), // Ogre Wrapper
    ("pt_short", 4.0), // Ogre Wrapper
    ("pt_string", 8.0), // Ogre Wrapper
    ("pt_unsigned_int", 3.0), // Ogre Wrapper
    ("pt_unsigned_long", 7.0), // Ogre Wrapper
    ("pt_unsigned_short", 5.0), // Ogre Wrapper
    ("pt_vector3", 9.0), // Ogre Wrapper
    ("r2_black", 1.0), // Стили логических операций
    ("r2_copypen", 13.0), // Стили логических операций
    ("r2_masknotpen", 3.0), // Стили логических операций
    ("r2_maskpen", 9.0), // Стили логических операций
    ("r2_maskpennot", 5.0), // Стили логических операций
    ("r2_mergenotpen", 12.0), // Стили логических операций
    ("r2_mergepen", 15.0), // Стили логических операций
    ("r2_mergepennot", 14.0), // Стили логических операций
    ("r2_nop", 11.0), // Стили логических операций
    ("r2_not", 6.0), // Стили логических операций
    ("r2_notcopypen", 4.0), // Стили логических операций
    ("r2_notmaskpen", 8.0), // Стили логических операций
    ("r2_notmergepen", 2.0), // Стили логических операций
    ("r2_notxorpen", 10.0), // Стили логических операций
    ("r2_white", 16.0), // Стили логических операций
    ("r2_xorpen", 7.0), // Стили логических операций
    ("sbf_dest_alpha", 6.0), // Ogre Wrapper
    ("sbf_dest_colour", 2.0), // Ogre Wrapper
    ("sbf_one", 0.0), // Ogre Wrapper
    ("sbf_one_minus_dest_alpha", 8.0), // Ogre Wrapper
    ("sbf_one_minus_dest_colour", 4.0), // Ogre Wrapper
    ("sbf_one_minus_source_alpha", 9.0), // Ogre Wrapper
    ("sbf_one_minus_source_colour", 5.0), // Ogre Wrapper
    ("sbf_source_alpha", 7.0), // Ogre Wrapper
    ("sbf_source_colour", 3.0), // Ogre Wrapper
    ("sbf_zero", 1.0), // Ogre Wrapper
    ("shadowtype_stencil_additive", 17.0), // Ogre Wrapper
    ("shadowtype_stencil_modulative", 18.0), // Ogre Wrapper
    ("shadowtype_texture_additive", 33.0), // Ogre Wrapper
    ("shadowtype_texture_modulative", 34.0), // Ogre Wrapper
    ("size_maxhide", 4.0), // Windows messages
    ("size_maximized", 2.0), // Windows messages
    ("size_maxshow", 3.0), // Windows messages
    ("size_minimized", 1.0), // Windows messages
    ("size_restored", 0.0), // Windows messages
    ("so_flat", 0.0), // Ogre Wrapper
    ("so_gouraud", 1.0), // Ogre Wrapper
    ("so_phong", 2.0), // Ogre Wrapper
    ("space3d", 8.0), // Инструменты
    ("string2d", 6.0), // Инструменты
    ("sw_hide", 0.0), // MESSAGEBOX constants
    ("sw_maximize", 3.0), // MESSAGEBOX constants
    ("sw_minimize", 6.0), // MESSAGEBOX constants
    ("sw_normal", 1.0), // MESSAGEBOX constants
    ("sw_restore", 9.0), // MESSAGEBOX constants
    ("sw_show", 5.0), // MESSAGEBOX constants
    ("sw_showmaximized", 3.0), // MESSAGEBOX constants
    ("sw_showminimized", 2.0), // MESSAGEBOX constants
    ("sw_showminnoactive", 7.0), // MESSAGEBOX constants
    ("sw_showna", 8.0), // MESSAGEBOX constants
    ("sw_shownoactivate", 4.0), // MESSAGEBOX constants
    ("sw_shownormal", 1.0), // MESSAGEBOX constants
    ("tex_type_1d", 1.0), // Ogre Wrapper
    ("tex_type_2d", 2.0), // Ogre Wrapper
    ("tex_type_3d", 3.0), // Ogre Wrapper
    ("tex_type_cube_map", 4.0), // Ogre Wrapper
    ("text2d", 7.0), // Инструменты
    ("texture3d", 9.0), // Инструменты
    ("vf_argument", 512.0), // флаги переменных класса
    ("vf_eqvar", 256.0), // флаги переменных класса
    ("vf_local", 2.0), // флаги переменных класса
    ("vf_return", 1024.0), // флаги переменных класса
    ("wm_allkeymessage", 1537.0), // Windows messages
    ("wm_allmousemessage", 1536.0), // Windows messages
    ("wm_canclose", 1538.0), // Windows messages
    ("wm_close", 16.0), // Windows messages
    ("wm_command", 273.0), // Windows messages
    ("wm_controlnotify", 1544.0), // Windows messages
    ("wm_destroy", 2.0), // Windows messages
    ("wm_getminmaxinfo", 36.0), // Windows messages
    ("wm_hyperjump", 1546.0), // Windows messages
    ("wm_keydown", 256.0), // Windows messages
    ("wm_keyup", 257.0), // Windows messages
    ("wm_lbuttondblclk", 515.0), // Windows messages
    ("wm_lbuttondown", 513.0), // Windows messages
    ("wm_lbuttonup", 514.0), // Windows messages
    ("wm_mbuttondblclk", 521.0), // Windows messages
    ("wm_mbuttondown", 519.0), // Windows messages
    ("wm_mbuttonup", 520.0), // Windows messages
    ("wm_mousemove", 512.0), // Windows messages
    ("wm_move", 3.0), // Windows messages
    ("wm_rbuttondblclk", 518.0), // Windows messages
    ("wm_rbuttondown", 516.0), // Windows messages
    ("wm_rbuttonup", 517.0), // Windows messages
    ("wm_size", 5.0), // Windows messages
    ("wm_spacedone", 1539.0), // Windows messages
    ("wm_spaceinit", 1540.0), // Windows messages
    ("word16", 2.0), // Streams
    ("word32", 4.0), // Streams
    ("word8", 0.0), // Streams
    ("ws_border", 8388608.0), // MCI Error codes
    ("ws_caption", 12582912.0), // MCI Error codes
    ("ws_child", 1073741824.0), // MCI Error codes
    ("ws_disabled", 134217728.0), // MCI Error codes
    ("ws_dlgframe", 4194304.0), // MCI Error codes
    ("ws_group", 131072.0), // MCI Error codes
    ("ws_hscroll", 1048576.0), // MCI Error codes
    ("ws_maximizebox", 65536.0), // MCI Error codes
    ("ws_minimizebox", 131072.0), // MCI Error codes
    ("ws_overlapped", 0.0), // MCI Error codes
    ("ws_popup", 2147483648.0), // MCI Error codes
    ("ws_sysmenu", 524288.0), // MCI Error codes
    ("ws_tabstop", 65536.0), // MCI Error codes
    ("ws_thickframe", 262144.0), // MCI Error codes
    ("ws_visible", 268435456.0), // MCI Error codes
    ("ws_vscroll", 2097152.0), // MCI Error codes
];

/// Значение константы по имени без учёта регистра.
pub fn lookup(name: &str) -> Option<f64> {
    let key = name.to_ascii_lowercase();
    CONSTANTS
        .binary_search_by(|(n, _)| (*n).cmp(key.as_str()))
        .ok()
        .map(|i| CONSTANTS[i].1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_known_constants() {
        assert_eq!(lookup("WM_LBUTTONDOWN"), Some(513.0));
        assert_eq!(lookup("wm_allmousemessage"), Some(1536.0));
        assert_eq!(lookup("PFC_MOVEOBJECT"), Some(32768.0));
        assert!((lookup("pi").unwrap() - std::f64::consts::PI).abs() < 1e-9);
        assert_eq!(lookup("нет такой"), None);
    }
}
