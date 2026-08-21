mod metrics;

pub(crate) use metrics::{
    main_page_project_path_width, main_page_tab_close_frame,
    main_page_tab_preferred_width_from_title_width,
    main_page_tab_preferred_width_from_title_width_with_close, main_page_tab_visible_cap_for_width,
    MAIN_PAGE_TAB_CHROME_SIDE_INSET, MAIN_PAGE_TAB_CLOSE_EXTENT, MAIN_PAGE_TAB_GAP,
    MAIN_PAGE_TAB_HEIGHT, MAIN_PAGE_TAB_MAX_WIDTH, MAIN_PAGE_TAB_MIN_WIDTH,
    MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH, MAIN_PAGE_TAB_OVERFLOW_WIDTH, MAIN_PAGE_TAB_STRIP_X,
    MAIN_PAGE_TAB_STRIP_Y, MAIN_PAGE_TAB_TITLE_FONT_SIZE,
};
