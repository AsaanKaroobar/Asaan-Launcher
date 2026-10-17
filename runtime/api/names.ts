//
//  names.ts
//  Project: Floe
//
//  Copyright (Floe) © 2026 René Jiménez
//  Licensed under the GNU AGPLv3

// The members of Raycast's Icon and Color enums, in the order @raycast/api 1.104.8 declares them.
// Extensions list them with Object.keys to build a picker; any other name still answers (see index.ts).
const numbers = Array.from({ length: 100 }, (_, index) => `Number${String(index).padStart(2, "0")}`);

export const iconNames = [
  ...`AddPerson Airplane AirplaneFilled AirplaneLanding AirplaneTakeoff Airpods Alarm AlarmRinging AlignCentre AlignLeft
AlignRight AmericanFootball Anchor AppWindow AppWindowGrid2x2 AppWindowGrid3x3 AppWindowList AppWindowSidebarLeft
AppWindowSidebarRight ArrowClockwise ArrowCounterClockwise ArrowDown ArrowDownCircle ArrowDownCircleFilled ArrowLeft
ArrowLeftCircle ArrowLeftCircleFilled ArrowNe ArrowRight ArrowRightCircle ArrowRightCircleFilled ArrowUp
ArrowUpCircle ArrowUpCircleFilled ArrowsContract ArrowsExpand AtSymbol BandAid BankNote BarChart BarCode BathTub
Battery BatteryCharging BatteryDisabled Bell BellDisabled Bike Binoculars Bird BlankDocument Bluetooth Boat Bold
Bolt BoltDisabled Book Bookmark Box Brush Bubble Bug Building BulletPoints BullsEye BullsEyeMissed Buoy Calculator
Calendar Camera Car Cart Cd Center Check CheckCircle CheckList CheckRosette Checkmark ChessPiece ChevronDown
ChevronDownSmall ChevronLeft ChevronLeftSmall ChevronRight ChevronRightSmall ChevronUp ChevronUpDown ChevronUpSmall
Circle CircleDisabled CircleEllipsis CircleFilled CircleProgress CircleProgress100 CircleProgress25 CircleProgress50
CircleProgress75 ClearFormatting Clipboard Clock Cloud CloudLightning CloudRain CloudSnow CloudSun Code CodeBlock
Cog Coin Coins CommandSymbol Compass ComputerChip Contrast CopyClipboard CreditCard CricketBall Crop Crown Crypto
DeleteDocument Desktop Devices Dna Document Dot Download Droplets Duplicate EditShape Eject Ellipsis
EllipsisVertical Emoji EmojiSad Envelope Eraser ExclamationMark Exclamationmark Exclamationmark2 Exclamationmark3
Eye EyeDisabled EyeDropper Female FilmStrip Filter Finder Fingerprint Flag Folder Footprints Forward ForwardFilled
FountainTip FullSignal GameController Gauge Gear Geopin Germ Gift Glasses Globe Goal Hammer HardDrive Hashtag
Heading Headphones Heart HeartDisabled Heartbeat Highlight Hourglass House Humidity Image Important Info Italics Key
Keyboard Layers Leaderboard Leaf LevelMeter LightBulb LightBulbOff LineChart Link List Livestream LivestreamDisabled
Lock LockDisabled LockUnlocked Logout Lorry Lowercase MagnifyingGlass Male Map Mask Maximize MedicalSupport
Megaphone MemoryChip MemoryStick Message Microphone MicrophoneDisabled Minimize Minus MinusCircle MinusCircleFilled
Mobile Monitor Moon MoonDown MoonUp Moonrise Mountain Mouse Move Mug MugSteam Multiply Music Network NewDocument
NewFolder`.split(/\s+/),
  ...numbers,
  ...`NumberList
Paperclip Paragraph Patch Pause PauseFilled Pencil Person PersonCircle PersonLines Phone PhoneRinging PieChart Pill
Pin PinDisabled Play PlayFilled Plug Plus PlusCircle PlusCircleFilled PlusMinusDivideMultiply PlusSquare
PlusTopRightSquare Power Print QuestionMark QuestionMarkCircle Quicklink QuotationMarks QuoteBlock Racket Raindrop
RaycastLogoNeg RaycastLogoPos Receipt Redo RemovePerson Repeat Replace ReplaceOne Reply Rewind RewindFilled Rocket
Rosette RotateAntiClockwise RotateClockwise Rss Ruler SaveDocument Shield ShortParagraph Shuffle Sidebar Signal0
Signal1 Signal2 Signal3 Snippets Snowflake SoccerBall Speaker SpeakerDown SpeakerHigh SpeakerLow SpeakerOff
SpeakerOn SpeakerUp SpeechBubble SpeechBubbleActive SpeechBubbleImportant SquareEllipsis StackedBars1 StackedBars2
StackedBars3 StackedBars4 Star StarCircle StarDisabled Stars Stop StopFilled Stopwatch Store StrikeThrough Sun
Sunrise Swatch Switch Syringe Tack TackDisabled Tag Temperature TennisBall Terminal Text TextCursor TextInput
TextSelection ThumbsDown ThumbsDownFilled ThumbsUp ThumbsUpFilled Ticket Torch Train Trash Tray Tree Trophy
TwoPeople Umbrella Underline Undo Upload Uppercase Video VideoDisabled Wallet Wand Warning Waveform Weights Wifi
WifiDisabled Wind Window Windsock WrenchScrewdriver WristWatch XMarkCircle XMarkCircleFilled XMarkCircleHalfDash
XMarkTopRightSquare Xmark TwoArrowsClockwise EyeSlash SpeakerArrowDown SpeakerArrowUp SpeakerSlash TextDocument
XmarkCircle`.split(/\s+/),
];

export const colorNames = ["Blue", "Green", "Magenta", "Orange", "Purple", "Red", "Yellow", "PrimaryText", "SecondaryText"];
