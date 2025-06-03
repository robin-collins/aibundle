// ==UserScript==
// @name         Extract All Colors from Webpage
// @namespace    http://tampermonkey.net/
// @version      1.0
// @description  Extracts all color references from a webpage (inline, stylesheets, CSS variables, computed styles)
// @author       OpenAI
// @match        *://*/*
// @grant        none
// ==/UserScript==

(function() {
    'use strict';

    // Strict color regex: hex, rgb(a), hsl(a), and color names
    const cssColorNames = [
        'aliceblue', 'antiquewhite', 'aqua', 'aquamarine', 'azure', 'beige', 'bisque', 'black', 'blanchedalmond', 'blue', 'blueviolet', 'brown', 'burlywood', 'cadetblue', 'chartreuse', 'chocolate', 'coral', 'cornflowerblue', 'cornsilk', 'crimson', 'cyan', 'darkblue', 'darkcyan', 'darkgoldenrod', 'darkgray', 'darkgreen', 'darkgrey', 'darkkhaki', 'darkmagenta', 'darkolivegreen', 'darkorange', 'darkorchid', 'darkred', 'darksalmon', 'darkseagreen', 'darkslateblue', 'darkslategray', 'darkslategrey', 'darkturquoise', 'darkviolet', 'deeppink', 'deepskyblue', 'dimgray', 'dimgrey', 'dodgerblue', 'firebrick', 'floralwhite', 'forestgreen', 'fuchsia', 'gainsboro', 'ghostwhite', 'gold', 'goldenrod', 'gray', 'green', 'greenyellow', 'grey', 'honeydew', 'hotpink', 'indianred', 'indigo', 'ivory', 'khaki', 'lavender', 'lavenderblush', 'lawngreen', 'lemonchiffon', 'lightblue', 'lightcoral', 'lightcyan', 'lightgoldenrodyellow', 'lightgray', 'lightgreen', 'lightgrey', 'lightpink', 'lightsalmon', 'lightseagreen', 'lightskyblue', 'lightslategray', 'lightslategrey', 'lightsteelblue', 'lightyellow', 'lime', 'limegreen', 'linen', 'magenta', 'maroon', 'mediumaquamarine', 'mediumblue', 'mediumorchid', 'mediumpurple', 'mediumseagreen', 'mediumslateblue', 'mediumspringgreen', 'mediumturquoise', 'mediumvioletred', 'midnightblue', 'mintcream', 'mistyrose', 'moccasin', 'navajowhite', 'navy', 'oldlace', 'olive', 'olivedrab', 'orange', 'orangered', 'orchid', 'palegoldenrod', 'palegreen', 'paleturquoise', 'palevioletred', 'papayawhip', 'peachpuff', 'peru', 'pink', 'plum', 'powderblue', 'purple', 'rebeccapurple', 'red', 'rosybrown', 'royalblue', 'saddlebrown', 'salmon', 'sandybrown', 'seagreen', 'seashell', 'sienna', 'silver', 'skyblue', 'slateblue', 'slategray', 'slategrey', 'snow', 'springgreen', 'steelblue', 'tan', 'teal', 'thistle', 'tomato', 'turquoise', 'violet', 'wheat', 'white', 'whitesmoke', 'yellow', 'yellowgreen'
    ];
    const colorNameSet = new Set(cssColorNames);
    const colorRegex = /#[0-9a-fA-F]{3,8}\b|rgba?\([^)]+\)|hsla?\([^)]+\)|\b(?:" + cssColorNames.join('|') + ")\b/gi;

    // Embedded color name map (hex lowercase → name, prefer w3cname=True)
    const colorNameMap = {
        '#000000': 'black',
        '#000001': 'its_still_basically_black',
        '#000002': 'still_black',
        '#000003': 'so_close_to_black_it_hurts',
        '#000007': 'double_o_seven',
        '#000009': 'really_dark_blue',
        '#00000a': 'minecraft_void',
        '#00000f': 'oof',
        '#000010': 'darkened_navy_blue',
        '#000030': 'edge_of_space',
        '#000042': 'the_answer',
        '#000056': 'void_heir',
        '#000069': 'nice_blue',
        '#000080': 'navy',
        '#00008b': 'darkblue',
        '#0000ad': 'what_would_jesus_blue',
        '#0000cd': 'mediumblue',
        '#0000ff': 'blue',
        '#000420': 'haha_nice',
        '#0004ff': 'sans',
        '#000666': 'devilishly_dark_blue',
        '#000da0': 'lyran_dasz',
        '#000dff': 'luna_blue',
        '#0011ff': 'im_blue_labadeelabeeda',
        '#0012ff': 'blueberry_benson',
        '#001337': 'leet_navy',
        '#002fa7': 'international_klein_blue',
        '#003153': 'prussian_blue',
        '#003b6f': 'tardis_blue',
        '#004182': 'vriska',
        '#004225': 'british_racing_green',
        '#0047ab': 'cobalt_blue',
        '#0051ff': 'im_blue_da_ba_dee_da',
        '#005682': 'arachnids_grip_cerulean',
        '#006400': 'darkgreen',
        '#0066ff': 'radix_blue',
        '#006900': 'nice_green',
        '#006969': 'nice_cyan',
        '#007bff': 'bootstrap_blue',
        '#007fff': 'azure',
        '#008000': 'green',
        '#008080': 'teal',
        '#0080ff': 'azure',
        '#008282': 'terezi',
        '#008740': 'deku',
        '#0088ff': 'zoom_online_meeting',
        '#008b8b': 'darkcyan',
        '#008cb4': 'mermaid_eyes',
        '#009001': 'over_9000_green',
        '#0099ff': 'spectre_blue',
        '#00a86b': 'jade',
        '#00aeff': 'groundspeed_blue',
        '#00b3ff': 'gatorade',
        '#00b6d5': 'weezer',
        '#00b7ff': 'oceans_horizon',
        '#00baba': 'baba_is_blue',
        '#00beef': 'beef_blue',
        '#00bfff': 'deepskyblue',
        '#00cccc': 'robins_egg_blue',
        '#00ccff': 'vivid_sky_blue',
        '#00ced1': 'darkturquoise',
        '#00ceff': 'hatsune_miku',
        '#00d1b2': 'bulma',
        '#00d5ff': 'sea_of_sky',
        '#00ddff': 'ocean_of_angels',
        '#00dead': 'lifeless_light',
        '#00e5ff': 'bouncy_blue',
        '#00e6ff': 'tropical_aqua_blue',
        '#00fa9a': 'mediumspringgreen',
        '#00ff00': 'lime',
        '#00ff7f': 'springgreen',
        '#00ffff': 'aqua',
        '#00ffff': 'cyan',
        '#010101': 'binary_black',
        '#010221': 'emperors_blue',
        '#011235': 'fibonacci_navy',
        '#012082': 'wallamaybungya_evening',
        '#012345': 'zero-indexed_blue',
        '#013370': 'hacker_blue',
        '#01b2ff': 'vexbloo',
        '#01cdfe': 'vaporwave_blue',
        '#020202': 'black_of_twos',
        '#024024': 'backwards_weed',
        '#030002': 'tokyo_black',
        '#030201': 'i_cant_believe_its_not_black',
        '#030303': 'off_black',
        '#0400ff': 'lapis_lipstick',
        '#040404': 'error_black',
        '#042069': 'epic_gamer_blue',
        '#04245c': 'its_not_a_phase_mom',
        '#046c43': 'dale_oeste',
        '#04be05': 'forest_hiking',
        '#04ff00': 'ultragreen',
        '#050505': 'five-degree_black',
        '#0595db': 'mendix_blue',
        '#0678be': 'drupal_blue',
        '#069420': 'epic_koosh_green',
        '#070407': 'its_quiet_uptown',
        '#0715cd': 'john_egbert_blue',
        '#0800ff': 'my_english_teachers_hair_color',
        '#080228': 'hello_darkness_my_old_friend',
        '#0a0555': 'ravenclaw',
        '#0a0a0a': 'jet_black',
        '#0abab5': 'tiffany_blue',
        '#0b0b0b': 'darth_vader',
        '#0ba2fd': 'cerebri_ai_blue',
        '#0c00ec': 'blue_screen_of_death',
        '#0c0c0c': 'hello_darkness_my_old_friend',
        '#0d00ff': 'ikea',
        '#0dead0': 'spectral_glow',
        '#0dff00': 'nickelodeon_slime',
        '#0f05c6': 'quite_blue',
        '#0ff0ff': 'off_cyan',
        '#100000': 'hundred_thousand',
        '#101010': 'binary_gray',
        '#102030': 'deca_navy',
        '#10ade1': 'jadinha',
        '#1100ff': 'blue_but_it_hurts_your_eyes',
        '#111111': 'dreamless_sleep',
        '#112233': 'seeing_double',
        '#112358': 'fibonacci_blue',
        '#114514': 'iiyokoiyo',
        '#11ff00': 'green_screen',
        '#120617': 'death',
        '#123123': 'test123',
        '#123321': 'greerg',
        '#123456': 'incremental_blue',
        '#123abc': 'alphanumeric_blue',
        '#124816': 'exponential_green',
        '#12e9a1': 'lelinha',
        '#12ff00': 'toxic_green',
        '#131072': 'blue_to_the_seventeenth',
        '#131313': 'cursed_black',
        '#133337': 'elite_teal',
        '#133700': 'elite_green',
        '#135791': 'odd_blue',
        '#140036': 'midnight',
        '#141414': 'a_distinctive_lack_of_hue',
        '#141421': 'the_square_root_of_blue',
        '#1500ff': 'ultramarine_blue',
        '#161222': 'my_sleep_paralysis_demon',
        '#16161d': 'eigengrau',
        '#161803': 'golden_mean_green',
        '#171717': 'cynical_black',
        '#177013': 'emergence',
        '#181818': 'black_olives',
        '#189bcc': 'weezer_blue_color',
        '#18f85b': 'lucky_amazing_green',
        '#1900b8': 'royal_blue',
        '#191919': 'rock_n_roll',
        '#191970': 'midnightblue',
        '#198964': 'tiananmen_tank',
        '#1a1a1a': 'la_la_la',
        '#1a2b3c': 'forward',
        '#1da1f2': 'twitter_blue',
        '#1db954': 'spotify_green',
        '#1e90ff': 'dodgerblue',
        '#1ecec3': 'minecraft_diamond',
        '#1eeb01': 'creeper_aw_man',
        '#1eff00': 'lucky_clover_shine',
        '#1f1e33': 'cametek',
        '#1f1f1f': 'engineer_black',
        '#200000': 'the_netherrack',
        '#202020': 'coronavirus_black',
        '#20b2aa': 'lightseagreen',
        '#211714': 'unus_annus',
        '#212121': 'defaultian_gray',
        '#2196f3': 'housecall',
        '#222b5d': 'roll_pen_ink_blue',
        '#224f76': 'incloud',
        '#228b22': 'forestgreen',
        '#22b7db': 'the_nice_pretty_blue',
        '#23272a': 'dark_discord',
        '#235711': 'forest_primes',
        '#23aacc': 'kilpatrick',
        '#246810': 'even_green',
        '#25ca61': 'terraria_green',
        '#2600ff': 'bluebolt',
        '#271828': 'eulers_aubergine',
        '#27ff27': 'smoov_green',
        '#2b2b2b': 'the_question',
        '#2c2f33': 'discord_gray',
        '#2cccd3': 'sparky_blue',
        '#2ce335': 'alarming_slime',
        '#2d2036': 'kristosaurus_purple',
        '#2e8b57': 'seagreen',
        '#2ec2a5': 'mintlodica',
        '#2f4f4f': 'darkslategray',
        '#2f4f4f': 'darkslategrey',
        '#2fff00': 'battery-powered',
        '#30184c': 'domolique_purlple',
        '#314159': 'blueberry_pi',
        '#314f45': 'flatulence_green',
        '#321123': 'ynagogany',
        '#323232': 'graphite',
        '#324369': 'denkfabrikblau',
        '#32cd32': 'limegreen',
        '#333333': 'dark_charcoal',
        '#336699': 'metafilter_blue',
        '#33aaff': 'datty',
        '#33ff00': 'highlighter_green',
        '#363636': 'moon_crater_gray',
        '#36393f': 'discord',
        '#3700ff': 'dragonblue',
        '#383838': 'cannon_smoke',
        '#393939': 'puro_grey',
        '#39c5bb': 'miku_green',
        '#3aa2c6': 'blue_raspberry_seed',
        '#3b0712': 'blood_of_my_enemies',
        '#3b56ff': 'scott_the_woz_blue',
        '#3b57ff': 'borderline_blue',
        '#3cb371': 'mediumseagreen',
        '#3d3d3d': '3d_black',
        '#3dba14': 'react_green',
        '#3e6535': 'watered_christmas_tree',
        '#3ef844': 'frog_zest',
        '#3fff00': 'neon_green',
        '#404040': 'error_404',
        '#40826d': 'viridian',
        '#4093ff': 'melis_blue',
        '#40e0d0': 'turquoise',
        '#40ff00': 'brighter_than_my_computer_screen',
        '#4169e1': 'royalblue',
        '#420000': 'dank_cordovan',
        '#420069': 'virgin_purple',
        '#420420': 'blaze_it_dark_magenta',
        '#420666': 'demon_weed',
        '#420690': 'nice_blaze',
        '#42069f': 'gamer_purple',
        '#420dab': 'certified_funny_color',
        '#424242': 'meaning_of_everything_gray',
        '#4267b2': 'facebook_blue',
        '#4285f4': 'google_blue',
        '#43bb9d': 'larry_stylinson',
        '#4400ff': 'bluple',
        '#444444': 'midnight_gray',
        '#450f4c': 'purple_eggplant',
        '#45fd23': 'dreamwastaken',
        '#4682b4': 'steelblue',
        '#4782c9': 'sansational',
        '#483d8b': 'darkslateblue',
        '#486d83': 'blue_loneliness',
        '#48d1cc': 'mediumturquoise',
        '#4940ff': 'pato_de_aqualand',
        '#4a8edf': 'drinkable_water',
        '#4ac925': 'harley_green',
        '#4b0082': 'indigo',
        '#4b4b4b': 'dusty_chimney',
        '#4d4d4d': 'time_gray',
        '#4d6780': 'denis',
        '#4dff00': 'tropical_uranium_green',
        '#500000': 'enemy_lines',
        '#505050': 'fiftieth_shade_of_grey',
        '#51a3e2': 'cadmiumcd_blue',
        '#51ff00': 'full_saturation_green',
        '#543210': 'liftoff_brown',
        '#54ddf1': 'harry_styles',
        '#555555': 'stone_cold_gray',
        '#556b2f': 'darkolivegreen',
        '#57058b': 'uwaterloo_engsoc_purple',
        '#5865f2': 'fake_blurple',
        '#5988ff': 'trench_squid',
        '#5a20a1': 'doofenshmirtz_evil_incorpoated',
        '#5ad5ad': 'sad_sad_teal',
        '#5bc2e7': 'zima_blue',
        '#5c94eb': 'pol_viejosabrozo',
        '#5f9ea0': 'cadetblue',
        '#600009': 'distant_lovers',
        '#6082b6': 'glaucous',
        '#610a0a': 'dried_blood',
        '#6200ff': 'nether_portal_indigo',
        '#626262': 'vantas_gray',
        '#6366f1': 'wasting_money_indigo',
        '#646464': 'light_charcoal',
        '#6495ed': 'cornflowerblue',
        '#64d968': 'wallamaybungya_pear',
        '#654321': 'decreasing_brown',
        '#660000': 'mississippi_state_university',
        '#661c39': 'alex',
        '#663399': 'rebeccapurple',
        '#666666': 'cursed_grey',
        '#66ccff': 'tianyi_blue',
        '#66cdaa': 'mediumaquamarine',
        '#66ff00': 'electric_neon_key_lime_pie',
        '#690000': 'nice_red',
        '#694200': 'heck_yeah',
        '#696969': 'dimgray',
        '#696969': 'dimgrey',
        '#69b00b': 'very_mature',
        '#6a5acd': 'slateblue',
        '#6b8e23': 'olivedrab',
        '#6e43a3': 'ace_purple',
        '#707070': 'gemini_gray',
        '#708090': 'slategray',
        '#708090': 'slategrey',
        '#71c269': 'policarpio_en_mal_dia',
        '#7289da': 'discord_blurple',
        '#728a51': 'bright_forest',
        '#7300ff': 'original_barney',
        '#737678': 'mountain_rock_grey',
        '#7393eb': 'sky_blue',
        '#73ff00': 'toxic_aliens',
        '#75b049': 'minecraft',
        '#777777': 'lucky_grey',
        '#778899': 'lightslategray',
        '#778899': 'lightslategrey',
        '#78b207': 'mike_wazowski_monsters_inc_green',
        '#78bde4': 'holiday_pool',
        '#7b00ff': 'electric_purple',
        '#7b5d92': 'ghostly_purple',
        '#7b68ee': 'mediumslateblue',
        '#7cfc00': 'lawngreen',
        '#7cfc0e': 'shrek_but_as_an_anime_girl',
        '#7d21a4': 'addictive_purple',
        '#7d7d7d': 'almost_charcoal',
        '#7f00ff': 'violet',
        '#7f7f7f': 'absolute_grey',
        '#7fff00': 'chartreuse',
        '#7fffd4': 'aquamarine',
        '#800000': 'maroon',
        '#800080': 'purple',
        '#800085': 'booobs',
        '#808000': 'olive',
        '#808080': 'gray',
        '#808080': 'grey',
        '#80ff00': 'lime',
        '#810081': 'ace_pride_purple',
        '#818a62': 'nagito_komaeda',
        '#820007': 'murder_but_with_style',
        '#842593': 'afton',
        '#854eae': 'thanos',
        '#861e93': 'kokichi',
        '#8684fa': 'oh_my_discord',
        '#876543': 'downward_brown',
        '#87ceeb': 'skyblue',
        '#87cefa': 'lightskyblue',
        '#88ff88': 'webwork_green',
        '#89f5e3': 'turquoise_pearl',
        '#8a0009': 'kiss_of_a_vampire',
        '#8a299a': 'purple_rain',
        '#8a2be2': 'blueviolet',
        '#8b0000': 'darkred',
        '#8b008b': 'darkmagenta',
        '#8b4513': 'saddlebrown',
        '#8be4c1': 'wallamaybungya_sea',
        '#8bf275': 'irish_spring_green_green',
        '#8fbc8f': 'darkseagreen',
        '#900000': 'tigers_blood',
        '#90ee90': 'lightgreen',
        '#911911': 'emergency_red',
        '#9370db': 'mediumpurple',
        '#9400d3': 'darkviolet',
        '#94c84c': 'lone_hunter_green',
        '#9500ff': 'star_platinum',
        '#958cff': 'gdragon',
        '#95ff00': 'neon_lime',
        '#964b00': 'brown',
        '#980505': 'elmo',
        '#987654': 'decreasing_beige',
        '#98b317': 'shrek',
        '#98fb98': 'palegreen',
        '#9900ff': 'perplexed_purple',
        '#9932cc': 'darkorchid',
        '#999999': 'million_grey',
        '#99aab5': 'greyple',
        '#99ff00': 'greenday',
        '#9acd32': 'yellowgreen',
        '#9b1414': 'bleeding_crimson',
        '#9b3b91': 'welcome_to_night_vale_purple',
        '#9e08be': 'purple_guy',
        '#9eb1ff': 'blue_but_it_drank_bleach',
        '#9fc1ed': 'louis_blue',
        '#a01a65': 'miles_edgeworth',
        '#a0522d': 'sienna',
        '#a10000': 'megido_red',
        '#a15000': 'adios_toreador',
        '#a1a100': 'captorial_citroid',
        '#a1a1a1': 'stale_steak',
        '#a1b2c3': 'alphanumerical_blue',
        '#a27ed5': 'bts',
        '#a2b3c5': 'kilpatricks_sadness',
        '#a2ff00': 'neon_highlighter_kid',
        '#a3ff00': 'radioactive_booger',
        '#a43ab9': 'i_hate_my_bridesmaids_pink',
        '#a52a2a': 'brown',
        '#a55a55': 'cheek_red',
        '#a60083': 'shivani_gakhar',
        '#a8ad18': 'leplep',
        '#a9a9a9': 'darkgray',
        '#a9a9a9': 'darkgrey',
        '#a9c7cf': 'ocean_eyes',
        '#aa00ff': 'enderdragon_breath',
        '#aaaaaa': 'screaming_grey',
        '#aabbcc': 'wood_pigeon',
        '#aae5a4': 'rule_34',
        '#aaf895': 'the_gayest_of_the_frogs',
        '#aaff00': 'mike_wazowski_on_acid',
        '#ab0a10': 'gryffindor',
        '#ab0b00': 'markiplier',
        '#ababab': 'shark',
        '#abc123': 'learning_green',
        '#abccba': 'palindrome_green',
        '#abcdef': 'alphabet_blue',
        '#acdc00': 'thunderstruck_neon',
        '#aceace': 'snake_eyes',
        '#ad0920': 'girl_in_red',
        '#adadad': 'advertisements',
        '#add8e6': 'lightblue',
        '#adff2f': 'greenyellow',
        '#ae00ff': 'emo_phase_hair_streak',
        '#aeaeae': 'polished_tin',
        '#afeeee': 'paleturquoise',
        '#b0000b': 'melon_red',
        '#b0c4de': 'lightsteelblue',
        '#b0e0e6': 'powderblue',
        '#b1000d': 'bloood',
        '#b200ff': 'msushi_purple',
        '#b22222': 'firebrick',
        '#b2228c': 'pinko',
        '#b2b200': 'cervantes_pineapple_yellow',
        '#b300ff': 'poison_purple_paradise',
        '#b40b92': 'youtube_en_2023',
        '#b4da55': 'badass',
        '#b500ff': 'lean',
        '#b536da': 'rose_lalonde_purple',
        '#b5d6c3': 'wallamaybungya_breeze',
        '#b76ecb': 'twitch_mareao',
        '#b8383b': 'red_team_spirit',
        '#b83c73': 'red_tedir',
        '#b8860b': 'darkgoldenrod',
        '#b89714': 'tarnished_trumpet',
        '#b898fb': 'bts',
        '#b8def5': 'gabriels_blue',
        '#b967ff': 'pepega_clap',
        '#b96d8e': 'pinky_pickle',
        '#ba0bab': 'purple_baobab',
        '#ba55d3': 'mediumorchid',
        '#baaaaa': 'sheep_gray',
        '#baabaa': 'white_sheep',
        '#baba15': 'baba_is_yellow',
        '#bababa': 'baby_talk',
        '#bada55': 'badass_green',
        '#badbad': 'naughty_boy',
        '#baebae': 'ice_ice',
        '#bb0f12': 'lextacy',
        '#bbbbbb': 'blyatful',
        '#bc8f8f': 'rosybrown',
        '#bcc0ef': 'rickroll_blue',
        '#bcddb3': 'a_manns_mint',
        '#bd0003': 'do_you_listen_to_girl_in_red',
        '#bdb76b': 'darkkhaki',
        '#beaded': 'beaded_purple',
        '#bebebe': 'tin_soldier',
        '#bedbed': 'cozy_comforter',
        '#beebee': 'do_you_like_jazz_blue',
        '#beeeee': 'bee_blue',
        '#beeeef': 'beef',
        '#beef00': 'rotten_beef',
        '#bf00ff': 'electric_purple',
        '#bfbbfb': 'liy',
        '#bfbfbf': 'nickel_grey',
        '#bfd100': 'dream_island_chartreuse',
        '#bffbff': 'friendship',
        '#bfff00': 'lime',
        '#c0c0c0': 'silver',
        '#c0fefe': 'covfefe_mint',
        '#c0ffee': 'arabica_mint',
        '#c170b5': 'clay_pink',
        '#c3e7fd': 'baby_blue',
        '#c3ff00': 'my_11_year_old_brothers_t_shirts',
        '#c42b76': 'karen',
        '#c4c4c4': 'explosive_grey',
        '#c521ac': 'lovely_fatima',
        '#c5c5c5': 'lunar_rock',
        '#c5f7fe': 'ranblue',
        '#c64fbe': 'unloaded_texture_purple',
        '#c6a097': 'tawanda_nyahuye',
        '#c71585': 'mediumvioletred',
        '#c8c8c8': 'grey_snow',
        '#c8ff00': 'liquid_neon',
        '#c9000d': 'among_us',
        '#cabcae': 'cabs_are_not_beige',
        '#cbcbcb': 'cerebellum_grey',
        '#cc0000': 'ussr_red',
        '#cc00ff': 'tinky_winky_teletubbies',
        '#cc5c5c': 'mazuat_red',
        '#cccccc': 'cerebral_grey',
        '#ccccff': 'periwinkle',
        '#cd0000': 'communist_red',
        '#cd5c5c': 'indianred',
        '#cd853f': 'peru',
        '#cdb7ec': 'ghostly_orchid',
        '#cdcdcd': 'compact_disc_grey',
        '#cee3e8': 'iceberg_that_hit_the_titanic',
        '#cf080b': 'depressed_crayon',
        '#cf2280': 'artificial_strawberry',
        '#cf7336': 'mann_co_orange',
        '#cfa4ae': 'dusty_rose',
        '#cfb5e2': 'lavender',
        '#cff0e8': 'squidwards_fragrance',
        '#d00d00': 'doodoo',
        '#d08439': 'cat',
        '#d0ac17': 'hamilton',
        '#d0d0d0': 'ancestral_water',
        '#d13652': 'cherry_pink',
        '#d2691e': 'chocolate',
        '#d2b48c': 'tan',
        '#d2bfff': 'vanilla_lavender_dream',
        '#d300ff': 'official_grayfruit_border',
        '#d33529': 'superficial_burn',
        '#d3d3d3': 'lightgray',
        '#d3d3d3': 'lightgrey',
        '#d4213d': 'polish_red',
        '#d600ff': 'bts',
        '#d64f81': 'love',
        '#d819f5': 'purple_lite',
        '#d8bfd8': 'thistle',
        '#d9c9fe': 'dusty_lavender',
        '#d9d9d9': 'foggy_mountain',
        '#da70d6': 'orchid',
        '#da8488': 'savory_salmon',
        '#daa520': 'goldenrod',
        '#dabad0': 'flintstone_pink',
        '#dabade': 'im_blue',
        '#dadada': 'dada_grey',
        '#daddad': 'dad',
        '#db31ab': 'andromada',
        '#db7093': 'palevioletred',
        '#dbdbdb': 'spider_silk',
        '#dc143c': 'crimson',
        '#dcdcdc': 'gainsboro',
        '#dda0dd': 'plum',
        '#ddd618': 'piss_yellow',
        '#dddddd': 'steam',
        '#dde2ec': 'nine_from_bfdi',
        '#de8e1f': 'cheems',
        '#dead00': 'deadly_yellow',
        '#deb887': 'burlywood',
        '#dec0b2': 'a_potato_flew_around_my_roooom',
        '#dec0de': 'cryptic_mauve',
        '#decaff': 'decaf_lavender',
        '#dedede': 'kingly_cloud',
        '#deedee': 'dexters_grey',
        '#e00707': 'strider_red',
        '#e0a2c6': 'harry_styles',
        '#e0e0e0': 'family_size_condensed_soup',
        '#e0ffff': 'lightcyan',
        '#e1e100': 'old_macdonalds_yellow',
        '#e1e1e1': 'cotton_grey',
        '#e23d28': 'swatchling_red',
        '#e3b4c4': 'ariana_grande',
        '#e3c0f7': 'yaoi_lord_yuzu',
        '#e3e3e3': 'electronic_entertainment_expo',
        '#e42b73': 'watermelon_sugar',
        '#e4e4e4': 'titanium_white',
        '#e5e1d0': 'polarbear_paws',
        '#e5e5e5': 'envelope_white',
        '#e5ff00': 'bumblebee_sun',
        '#e600ff': 'booshy_grapes',
        '#e62100': 'furry_orange',
        '#e62169': 'hot_furry_pink',
        '#e6e6e6': 'extraordinary_abundance_of_tinge',
        '#e6e6fa': 'lavender',
        '#e73b3d': 'wallamaybungya_red',
        '#e74c3c': 'tenacious_cinnabar',
        '#e7b53b': 'australium',
        '#e83821': 'ruben_red',
        '#e89210': 'patty_blinger',
        '#e8ffd4': 'sad_sunshine_yellow',
        '#e93159': 'wallamaybungya_pink',
        '#e9967a': 'darksalmon',
        '#e9b413': 'trumpet_gold',
        '#ea8220': 'haxe_orange',
        '#eaeaea': 'ea_white',
        '#eb3318': 'thermic_orange',
        '#eb4518': 'reddit_moment',
        '#ebfffc': 'arctic_glacier',
        '#ec008b': 'the_1st_girl',
        '#ed0e0b': 'pol_apasionado',
        '#ed0f87': 'danganronpa_blood',
        '#edd32b': 'wilbur_soot',
        '#ededed': 'ed_ed_n_eddy',
        '#ee00ff': 'headache_inducing_pink',
        '#ee362e': 'cant_even_red',
        '#ee6130': 'brainzooming_orange_blast',
        '#ee82ee': 'violet',
        '#eee8aa': 'palegoldenrod',
        '#eeeeee': 'screeching_white',
        '#eeff00': 'zeuss_bolt',
        '#ef0035': 'awesome_car',
        '#ef2f08': 'emergency_meeting_red',
        '#ef3340': 'singapore_red',
        '#ef94aa': 'technoblade',
        '#f0000d': 'hungry_red',
        '#f0509a': 'body_discovery_announcement',
        '#f08080': 'lightcoral',
        '#f0e68c': 'khaki',
        '#f0f8ff': 'aliceblue',
        '#f0fff0': 'honeydew',
        '#f0ffff': 'azure',
        '#f1d96e': 'mocha_dandelion',
        '#f1f1f1': 'paper',
        '#f20000': 'scarlet_king',
        '#f20707': 'roblox',
        '#f2a400': 'dirk_orange',
        '#f2efe1': 'bones_are_actually_wet',
        '#f2ff00': 'spongebob',
        '#f39200': 'kevert_narancs',
        '#f42069': 'haha_nice_pink',
        '#f4a460': 'sandybrown',
        '#f4e32a': 'staffy_yellow',
        '#f4ff00': 'busy_bee_yellow',
        '#f520c8': 'hottest_of_pinks',
        '#f5821f': 'cloudflare_orange',
        '#f5dc00': 'hufflepuff',
        '#f5deb3': 'wheat',
        '#f5f5dc': 'beige',
        '#f5f5f5': 'whitesmoke',
        '#f5fffa': 'mintcream',
        '#f5fffb': 'smooth_pearl',
        '#f5ffff': 'ivory_wedding',
        '#f63e75': 'truenamelcoche',
        '#f6ff00': 'brighter_than_my_future',
        '#f700ff': 'magenta_with_personality',
        '#f79400': 'trump',
        '#f7ea48': 'pupa',
        '#f7f7f7': 'lynx_white',
        '#f7faff': 'crying_on_the_bathroom_floor',
        '#f7fdff': 'in_a_distant_dream',
        '#f7fffa': 'white_but_its_not',
        '#f7fffc': 'the_valentino_white_bag',
        '#f86092': 'harry_styles',
        '#f8edff': 'dreams_in_lavender',
        '#f8f8ff': 'ghostwhite',
        '#f900f1': 'peluca_rosa_de_pol',
        '#f9f9f9': 'ivory_egg',
        '#fa54bb': 'a_pink_phenomenon',
        '#fa8072': 'salmon',
        '#fa8383': 'picnic_in_a_strawberry_field',
        '#facade': 'facade',
        '#face00': 'sick_face',
        '#face00': 'sick_face',
        '#facfac': 'heck',
        '#fad314': 'atomic_yellow',
        '#fade00': 'fade_yellow',
        '#fae100': 'sunshine',
        '#faebd7': 'antiquewhite',
        '#faf0e6': 'linen',
        '#fafad2': 'lightgoldenrodyellow',
        '#fafafa': 'the_official_grayfruit_sheen',
        '#fafffc': 'transcended',
        '#fafffd': 'clouds_from_heaven',
        '#faffff': 'foggy_glasses',
        '#fb00ff': 'a_unicorn_sneeze',
        '#fb1fb1': 'fbi',
        '#fb9700': 'solnishkos_sunrise',
        '#fbf7ff': 'nintendo_wii',
        '#fbfaff': 'almost_there_white',
        '#fbfbfb': 'whiteout',
        '#fbff00': 'electric_banana',
        '#fbfff7': 'colgate_white',
        '#fce0e2': 'taylor_swift',
        '#fce300': 'bandito_yellow',
        '#fcfaff': 'fluffy_ice',
        '#fcfcfc': 'snowflake',
        '#fcfffa': 'egg',
        '#fcffff': 'polar_bear_in_a_blizzard',
        '#fd0101': 'karkat',
        '#fd6c22': 'oppositelock_orange',
        '#fdca9c': 'peachy_keen',
        '#fdf5e6': 'oldlace',
        '#fdf7ff': 'i_think_this_is_called_white',
        '#fdf901': 'delayed_yellow',
        '#fdfcff': 'angelwings',
        '#fdfdfd': 'brilliance',
        '#fdfff5': 'milk',
        '#fdfff7': 'quarantined_cream',
        '#fe0000': 'simply_red',
        '#fe90ff': 'stan_loona',
        '#fed7f6': 'strawberry_flavoured_yogurt',
        '#fedfba': 'backwards_beige',
        '#feed00': 'feed_me_yellow',
        '#feeded': 'incorrect_grammar_pink',
        '#feefee': 'double_dues',
        '#fef4dd': 'iced_almond',
        '#fef65b': 'dodie_yellow',
        '#fef7ff': 'james_charless_foundation',
        '#fefcff': 'mom_we_have_white_at_home',
        '#fefefe': 'white_as_heaven',
        '#feffeb': 'fresh_milk',
        '#fefffc': 'flashback_mary',
        '#feffff': 'snowdrop',
        '#ff0000': 'red',
        '#ff0004': 'cherry_soda_red',
        '#ff0008': 'marinara_red',
        '#ff0011': 'e-boys_led_lights',
        '#ff0015': 'british_phone_box_red',
        '#ff0022': 'weee_ooooh_im_a_firetruck',
        '#ff0037': 'the_blood_of_your_enemies',
        '#ff0040': 'juicy_watermelon',
        '#ff0051': 'hawaiian_raspberry',
        '#ff007f': 'rose',
        '#ff00aa': 'danganronpa_blood',
        '#ff00dd': 'jazzercise_leg_warmers',
        '#ff00e1': 'the_pinkest_pink',
        '#ff00ff': 'fuchsia',
        '#ff00ff': 'magenta',
        '#ff0800': 'the_blood_of_my_enemies',
        '#ff0d00': 'shocking_crimson',
        '#ff1100': 'netflix',
        '#ff1493': 'deeppink',
        '#ff1500': 'carolina_reaper',
        '#ff1a1a': 'mario',
        '#ff1e1e': 'communism',
        '#ff2f00': 'red_riot',
        '#ff4000': 'phoenixs_feather',
        '#ff4242': 'realmy_red',
        '#ff4400': 'inferno_orange',
        '#ff4500': 'orangered',
        '#ff5500': 'cheeto_dusttt',
        '#ff58e3': 'official_grayfruit_pink',
        '#ff6347': 'tomato',
        '#ff6500': 'hacker_news_orange',
        '#ff69b4': 'hotpink',
        '#ff6a00': 'cheetos',
        '#ff7300': 'pornhub_orange',
        '#ff7e30': 'channel_orange',
        '#ff7f50': 'coral',
        '#ff8000': 'orange',
        '#ff8800': 'oompa_loompa_type_beat',
        '#ff8c00': 'darkorange',
        '#ff9100': 'its_nerf_or_nothin',
        '#ff9500': 'cheeto_dusty_orange',
        '#ff96ef': 'kirby',
        '#ff9900': 'vitamin_c',
        '#ffa07a': 'lightsalmon',
        '#ffa500': 'orange',
        '#ffb6c1': 'lightpink',
        '#ffbb00': 'nacho_cheese',
        '#ffbf00': 'amber',
        '#ffc0cb': 'pink',
        '#ffc400': 'hamilton',
        '#ffce0a': 'our_kicks',
        '#ffd000': 'cheese',
        '#ffd1d9': 'pink_lemonade',
        '#ffd5c6': 'pink_lemonade',
        '#ffd700': 'gold',
        '#ffdab9': 'peachpuff',
        '#ffdb4b': 'mypvp_yellow',
        '#ffdbdd': 'cotton_candy',
        '#ffdead': 'navajowhite',
        '#ffe100': 'mcdonalds_arches',
        '#ffe4b5': 'moccasin',
        '#ffe4c4': 'bisque',
        '#ffe4e1': 'mistyrose',
        '#ffe500': 'soltorg_yellow',
        '#ffe600': 'lemonade_with_sugar',
        '#ffebcd': 'blanchedalmond',
        '#ffefd5': 'papayawhip',
        '#fff0f5': 'lavenderblush',
        '#fff2fa': 'candyfloss_dream',
        '#fff3db': 'salted_caramel_vanilla_milkshake',
        '#fff400': 'ikea_bird_yellow',
        '#fff5ee': 'seashell',
        '#fff5f5': 'discord_light_mode',
        '#fff700': 'spongebob_yellow',
        '#fff7f7': 'cloud_paint',
        '#fff8dc': 'cornsilk',
        '#fff8e7': 'cosmic_latte',
        '#fff8f0': 'parents_divorce_papers',
        '#fff9ed': 'fluffy_cream',
        '#fff9f2': 'coconut_milk_white',
        '#fffacd': 'lemonchiffon',
        '#fffaf0': 'floralwhite',
        '#fffaf2': 'bone',
        '#fffaf5': 'boxyfresh',
        '#fffafa': 'snow',
        '#fffafc': 'pink_tint_white',
        '#fffafd': 'you_can_barely_tell_its_pink',
        '#fffaff': 'your_phone_at_3am',
        '#fffb00': 'berry_bee_benson',
        '#fffbf0': 'vanilla_shake',
        '#fffbf5': 'powder_pearl',
        '#fffbfa': 'whiterose',
        '#fffc00': 'snapchat_yellow',
        '#fffcfe': 'salt',
        '#fffcff': 'i_couldnt_get_it_exactly_white',
        '#fffdfa': 'egg',
        '#fffdfc': 'not-quite_not-white',
        '#fffef5': 'coconut_creme_pie',
        '#fffefa': 'wedding_dress',
        '#fffefc': 'ceo_of_white',
        '#fffefe': 'still_white',
        '#fffeff': 'stormtrooper',
        '#ffff00': 'yellow',
        '#ffff57': 'psychic_friend_fred_bear',
        '#ffffe0': 'lightyellow',
        '#fffff0': 'ivory',
        '#fffff1': 'raw_marshmallow',
        '#fffff5': 'french_vanilla_sunrise',
        '#fffffa': 'flashbackmary',
        '#ffffff': 'white',
    };

    // Helper: Convert color name to rgb string
    function nameToRGB(name) {
        // Use a dummy element to resolve computed color
        const el = document.createElement('div');
        el.style.color = name;
        document.body.appendChild(el);
        const rgb = getComputedStyle(el).color;
        document.body.removeChild(el);
        return rgb;
    }

    // Helper: Map rgb/rgba/hex to name if possible
    function colorValueToName(value) {
        let rgb;
        let hex = null;
        if (value.startsWith('#')) {
            hex = value.toLowerCase();
            rgb = hexToRGB(hex);
        } else if (value.startsWith('rgb') || value.startsWith('hsl')) {
            // Use a dummy element to resolve
            const el = document.createElement('div');
            el.style.color = value;
            document.body.appendChild(el);
            rgb = getComputedStyle(el).color;
            document.body.removeChild(el);
            // Try to get hex from computed rgb
            hex = rgbStringToHex(rgb);
        } else if (colorNameSet.has(value.toLowerCase())) {
            rgb = nameToRGB(value);
            hex = rgbStringToHex(rgb);
        }
        // Try custom colorNameMap first
        if (hex && colorNameMap[hex]) return colorNameMap[hex];
        // Fallback to CSS names
        for (const name of cssColorNames) {
            if (nameToRGB(name).toLowerCase() === rgb.toLowerCase()) {
                return name;
            }
        }
        return null;
    }

    // Helper: Convert hex to rgb string
    function hexToRGB(hex) {
        let c = hex.replace('#', '');
        if (c.length === 3) c = c[0] + c[0] + c[1] + c[1] + c[2] + c[2];
        if (c.length === 4) c = c[0] + c[0] + c[1] + c[1] + c[2] + c[2] + c[3] + c[3];
        if (c.length === 6) c = c + 'ff';
        if (c.length === 8) {
            const r = parseInt(c.slice(0, 2), 16);
            const g = parseInt(c.slice(2, 4), 16);
            const b = parseInt(c.slice(4, 6), 16);
            const a = parseInt(c.slice(6, 8), 16) / 255;
            if (a === 1) return `rgb(${r}, ${g}, ${b})`;
            return `rgba(${r}, ${g}, ${b}, ${a.toFixed(3).replace(/\.0+$/, '')})`;
        }
        return hex;
    }

    // Helper: Convert rgb string to hex
    function rgbStringToHex(rgb) {
        const m = rgb.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
        if (!m) return null;
        let r = parseInt(m[1]),
            g = parseInt(m[2]),
            b = parseInt(m[3]);
        let a = m[4] !== undefined ? Math.round(parseFloat(m[4]) * 255) : 255;
        return '#' + [r, g, b, a].map(x => x.toString(16).padStart(2, '0')).join('').replace(/ff$/, '');
    }

    // Extract colors from a string
    function extractColorsFromString(str) {
        if (!str) return [];
        return Array.from(str.matchAll(colorRegex), m => m[0]);
    }

    // Helper: Get all stylesheets (internal and external)
    function getAllCSSRules() {
        let rules = [];
        for (let sheet of document.styleSheets) {
            try {
                if (sheet.cssRules) {
                    for (let rule of sheet.cssRules) {
                        rules.push(rule);
                    }
                }
            } catch (e) {
                // Cross-origin stylesheet, skip
            }
        }
        return rules;
    }

    // 1. Inline styles
    function extractInlineColors() {
        const colors = new Set();
        document.querySelectorAll('[style]').forEach(el => {
            const style = el.getAttribute('style');
            extractColorsFromString(style).forEach(c => colors.add(c));
        });
        return colors;
    }

    // 2. Internal and external stylesheets
    function extractStylesheetColors() {
        const colors = new Set();
        const rules = getAllCSSRules();
        for (let rule of rules) {
            if (rule.style) {
                for (let i = 0; i < rule.style.length; i++) {
                    let value = rule.style.getPropertyValue(rule.style[i]);
                    extractColorsFromString(value).forEach(c => colors.add(c));
                }
            }
        }
        return colors;
    }

    // 3. CSS Variables
    function extractCSSVariables() {
        const vars = new Set();
        const rules = getAllCSSRules();
        for (let rule of rules) {
            if (rule.style) {
                for (let i = 0; i < rule.style.length; i++) {
                    let prop = rule.style[i];
                    if (prop.startsWith('--')) {
                        let value = rule.style.getPropertyValue(prop);
                        extractColorsFromString(value).forEach(c => vars.add(`${prop}: ${c}`));
                    }
                }
            }
        }
        // Also check inline styles for CSS variables
        document.querySelectorAll('[style]').forEach(el => {
            const style = el.style;
            for (let i = 0; i < style.length; i++) {
                let prop = style[i];
                if (prop.startsWith('--')) {
                    let value = style.getPropertyValue(prop);
                    extractColorsFromString(value).forEach(c => vars.add(`${prop}: ${c}`));
                }
            }
        });
        return vars;
    }

    // 4. Computed styles from all visible elements
    function extractComputedColors() {
        const colorProps = [
            'color', 'background-color', 'border-color', 'border-top-color', 'border-right-color',
            'border-bottom-color', 'border-left-color', 'outline-color', 'text-decoration-color',
            'column-rule-color', 'fill', 'stroke'
        ];
        const colors = new Set();
        document.querySelectorAll('*').forEach(el => {
            if (el.offsetParent !== null) { // visible elements
                const style = getComputedStyle(el);
                colorProps.forEach(prop => {
                    let value = style.getPropertyValue(prop);
                    extractColorsFromString(value).forEach(c => colors.add(c));
                });
            }
        });
        return colors;
    }

    // Enhanced: Estimate color usage by area
    function extractAllColorsWithUsage() {
        const colorAreas = {};
        const colorMeta = {};
        let totalArea = 0;
        // Helper to add area for a color
        function addArea(color, area, meta) {
            if (!color) return;
            if (!colorAreas[color]) colorAreas[color] = 0;
            colorAreas[color] += area;
            if (!colorMeta[color]) colorMeta[color] = meta;
        }
        // Walk all visible elements
        document.querySelectorAll('*').forEach(el => {
            if (el.offsetParent === null) return; // not visible
            const rect = el.getBoundingClientRect();
            if (rect.width === 0 || rect.height === 0) return;
            const area = rect.width * rect.height;
            totalArea += area;
            // Background color
            let bg = getComputedStyle(el).backgroundColor;
            if (bg && bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent') {
                addArea(bg, area, { type: 'background' });
            }
            // Text color
            let tc = getComputedStyle(el).color;
            if (tc && tc !== 'rgba(0, 0, 0, 0)' && tc !== 'transparent') {
                addArea(tc, area, { type: 'text' });
            }
        });
        // Now, format and filter as before, but with area info
        const formatted = [];
        const seen = new Set();
        for (const color in colorAreas) {
            let value = color.trim();
            let name = null;
            let hex = null;
            let rgb = null;
            // Only keep valid color values
            if (/^#[0-9a-fA-F]{3,8}$/.test(value)) {
                hex = value.toLowerCase();
                rgb = hexToRGB(hex);
            } else if (value.startsWith('rgb') || value.startsWith('hsl')) {
                rgb = value.startsWith('hsl') ? nameToRGB(value) : value;
                hex = rgbStringToHex(rgb);
            } else if (colorNameSet.has(value.toLowerCase())) {
                rgb = nameToRGB(value);
                hex = rgbStringToHex(rgb);
            }
            name = colorNameMap[hex] || colorValueToName(value);
            // Format: [percent]% #hexcode rgb(x, x, x) -- name
            let percent = ((colorAreas[color] / totalArea) * 100).toFixed(2);
            let out = null;
            if (hex && rgb) {
                out = `${percent}% #${hex.replace(/^#/, '')} ${rgb}`;
                if (name) out += ` -- ${name}`;
            } else if (rgb) {
                out = `${percent}% ${rgb}`;
                if (name) out += ` -- ${name}`;
            } else if (hex) {
                out = `${percent}% #${hex.replace(/^#/, '')}`;
                if (name) out += ` -- ${name}`;
            } else {
                out = `${percent}% ${value}`;
            }
            if (!seen.has(out)) {
                formatted.push({ out, percent: parseFloat(percent), color, hex });
                seen.add(out);
            }
        }
        // Sort by percent descending
        formatted.sort((a, b) => b.percent - a.percent);
        return formatted;
    }

    // Modal creation
    function showModalWithUsage() {
        const colorObjs = extractAllColorsWithUsage();
        const colors = colorObjs.map(obj => obj.out);
        // Remove existing modal if present
        const oldModal = document.getElementById('color-extract-modal');
        if (oldModal) oldModal.remove();

        // Modal overlay
        const overlay = document.createElement('div');
        overlay.id = 'color-extract-modal';
        overlay.style.position = 'fixed';
        overlay.style.top = '0';
        overlay.style.left = '0';
        overlay.style.width = '100vw';
        overlay.style.height = '100vh';
        overlay.style.background = 'rgba(0,0,0,0.35)';
        overlay.style.zIndex = '999999';
        overlay.style.display = 'flex';
        overlay.style.alignItems = 'center';
        overlay.style.justifyContent = 'center';

        // Modal box
        const modal = document.createElement('div');
        modal.style.background = '#fff';
        modal.style.borderRadius = '12px';
        modal.style.boxShadow = '0 4px 32px rgba(0,0,0,0.18)';
        modal.style.padding = '2em 2em 1.5em 2em';
        modal.style.maxWidth = '1200px'; // wider modal
        modal.style.width = 'min(95vw, 1200px)';
        modal.style.maxHeight = '80vh';
        modal.style.overflow = 'auto';
        modal.style.position = 'relative';
        modal.style.minWidth = '320px';

        // Close button
        const closeBtn = document.createElement('button');
        closeBtn.innerHTML = '&times;';
        closeBtn.style.position = 'absolute';
        closeBtn.style.top = '12px';
        closeBtn.style.right = '16px';
        closeBtn.style.background = 'none';
        closeBtn.style.border = 'none';
        closeBtn.style.fontSize = '2em';
        closeBtn.style.cursor = 'pointer';
        closeBtn.style.color = '#888';
        closeBtn.title = 'Close';
        closeBtn.onclick = () => overlay.remove();
        modal.appendChild(closeBtn);

        // Title
        const title = document.createElement('h2');
        title.textContent = 'Extracted Colors (sorted by estimated screen usage)';
        title.style.marginTop = '0';
        title.style.marginBottom = '0.5em';
        title.style.fontSize = '1.4em';
        modal.appendChild(title);

        // Note
        const note = document.createElement('div');
        note.textContent = 'Percentages are approximate and may overcount due to overlapping elements and text area estimation.';
        note.style.fontSize = '0.95em';
        note.style.color = '#666';
        note.style.marginBottom = '1em';
        modal.appendChild(note);

        // Copy button with icon
        const copyBtn = document.createElement('button');
        copyBtn.style.display = 'flex';
        copyBtn.style.alignItems = 'center';
        copyBtn.style.gap = '0.5em';
        copyBtn.style.background = '#f5f5f5';
        copyBtn.style.border = '1px solid #ccc';
        copyBtn.style.borderRadius = '6px';
        copyBtn.style.padding = '0.4em 1em';
        copyBtn.style.cursor = 'pointer';
        copyBtn.style.fontSize = '1em';
        copyBtn.style.marginBottom = '1em';
        copyBtn.title = 'Copy color list to clipboard';
        // SVG copy icon
        copyBtn.innerHTML = `<svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="6" y="6" width="9" height="9" rx="2" stroke="#333" stroke-width="1.5"/><rect x="4" y="4" width="9" height="9" rx="2" fill="#fff" stroke="#333" stroke-width="1.5"/></svg> <span>Copy</span>`;
        copyBtn.onclick = () => {
            const text = colors.join('\n');
            navigator.clipboard.writeText(text).then(() => {
                copyBtn.querySelector('span').textContent = 'Copied!';
                setTimeout(() => {
                    copyBtn.querySelector('span').textContent = 'Copy';
                }, 1200);
            });
        };
        modal.appendChild(copyBtn);

        // Color list
        const list = document.createElement('ul');
        list.style.listStyle = 'none';
        list.style.padding = '0';
        list.style.margin = '0';
        list.style.maxHeight = '50vh';
        list.style.overflowY = 'auto';
        list.style.fontSize = '0.92em'; // smaller text
        list.style.lineHeight = '1.3';
        colorObjs.forEach(obj => {
            const c = obj.out;
            const li = document.createElement('li');
            li.style.display = 'flex';
            li.style.alignItems = 'center';
            li.style.gap = '0.7em';
            li.style.marginBottom = '0.2em';
            // Color swatch
            const swatch = document.createElement('span');
            swatch.style.display = 'inline-block';
            swatch.style.width = '1.2em';
            swatch.style.height = '1.2em';
            swatch.style.border = '1px solid #ccc';
            swatch.style.borderRadius = '3px';
            let colorVal = obj.hex ? obj.hex : (c.match(/#([0-9a-f]{3,8})/i) ? `#${c.match(/#([0-9a-f]{3,8})/i)[1]}` : c.split('--')[0].trim().split(' ')[0]);
            swatch.style.background = colorVal;
            swatch.title = c;
            li.appendChild(swatch);
            // Color text
            const text = document.createElement('span');
            text.textContent = c;
            text.style.fontFamily = 'monospace';
            text.style.fontSize = '0.92em';
            li.appendChild(text);
            list.appendChild(li);
        });
        modal.appendChild(list);

        // Dismiss modal on overlay click (but not when clicking inside modal)
        overlay.addEventListener('click', e => {
            if (e.target === overlay) overlay.remove();
        });

        overlay.appendChild(modal);
        document.body.appendChild(overlay);
    }

    // Floating button creation (update to use new modal)
    function createFloatingButton() {
        const oldBtn = document.getElementById('color-extract-fab');
        if (oldBtn) oldBtn.remove();
        const btn = document.createElement('button');
        btn.id = 'color-extract-fab';
        btn.title = 'Extract colors from this page';
        btn.style.position = 'fixed';
        btn.style.top = '20px';
        btn.style.right = '20px';
        btn.style.zIndex = '1000000';
        btn.style.width = '48px';
        btn.style.height = '48px';
        btn.style.borderRadius = '50%';
        btn.style.background = 'linear-gradient(135deg, #f7971e 0%, #ffd200 100%)';
        btn.style.boxShadow = '0 2px 8px rgba(0,0,0,0.18)';
        btn.style.border = 'none';
        btn.style.display = 'flex';
        btn.style.alignItems = 'center';
        btn.style.justifyContent = 'center';
        btn.style.cursor = 'pointer';
        btn.style.padding = '0';
        btn.style.transition = 'box-shadow 0.2s';
        btn.style.outline = 'none';
        btn.style.opacity = '0.92';
        btn.onmouseenter = () => btn.style.opacity = '1';
        btn.onmouseleave = () => btn.style.opacity = '0.92';
        btn.onmousedown = () => btn.style.boxShadow = '0 1px 4px rgba(0,0,0,0.22)';
        btn.onmouseup = () => btn.style.boxShadow = '0 2px 8px rgba(0,0,0,0.18)';
        btn.innerHTML = `<svg width="28" height="28" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg"><circle cx="14" cy="14" r="12" fill="#fff" stroke="#333" stroke-width="2"/><circle cx="9.5" cy="10" r="2" fill="#f7971e"/><circle cx="18.5" cy="10" r="2" fill="#ffd200"/><circle cx="10" cy="17" r="2" fill="#a1c45a"/><circle cx="18" cy="17" r="2" fill="#5ac4b1"/></svg>`;
        btn.addEventListener('click', showModalWithUsage);
        document.body.appendChild(btn);
    }

    // Run after DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', createFloatingButton);
    } else {
        createFloatingButton();
    }
})();