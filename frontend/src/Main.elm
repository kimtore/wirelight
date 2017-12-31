module Main exposing (..)

import Html exposing (Html, text, div, h1, h2, h3, input, select, option)
import Html.Attributes exposing (src, type_, class, min, max, value)
import Html.Events exposing (onInput)
import Json.Encode
import WebSocket


---- MODEL ----


type alias Effect =
    { key : String
    , name : String
    }


type alias Model =
    { effect : String
    , hue : ColorValue
    , chroma : ColorValue
    , luminance : ColorValue
    , serverState : String
    }


type alias ColorValue =
    Int


init : ( Model, Cmd Msg )
init =
    ( { effect = "off"
      , hue = 0
      , chroma = 0
      , luminance = 0
      , serverState = ""
      }
    , Cmd.none
    )


effects : List Effect
effects =
    [ { key = "off", name = "Darkness" }
    , { key = "solid", name = "Solid color" }
    , { key = "northernLights", name = "Northern lights" }
    , { key = "wave", name = "Wave" }
    ]



---- UPDATE ----


type HclParam
    = Hue
    | Chroma
    | Luminance


type Msg
    = NoOp
    | SendColors
    | ServerState String
    | EffectChange String
    | HclChange HclParam String


colorFraction : ColorValue -> Float
colorFraction c =
    toFloat c / 65535


colorFractionString : ColorValue -> String
colorFractionString c =
    String.left 6 (toString (colorFraction c))


zint : String -> ColorValue
zint s =
    Result.withDefault 0 (String.toInt s)


colorObject : Model -> Json.Encode.Value
colorObject m =
    Json.Encode.object
        [ ( "effect", Json.Encode.string m.effect )
        , ( "hue", Json.Encode.int m.hue )
        , ( "chroma", Json.Encode.int m.chroma )
        , ( "luminance", Json.Encode.int m.luminance )
        ]


sendColors : Model -> Cmd Msg
sendColors model =
    WebSocket.send "ws://nova:8011/" (Json.Encode.encode 2 (colorObject model))


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        HclChange Hue s ->
            update SendColors { model | hue = zint s }

        HclChange Chroma s ->
            update SendColors { model | chroma = zint s }

        HclChange Luminance s ->
            update SendColors { model | luminance = zint s }

        EffectChange s ->
            update SendColors { model | effect = s }

        SendColors ->
            ( model, sendColors model )

        ServerState s ->
            ( { model | serverState = s }, Cmd.none )

        NoOp ->
            ( model, Cmd.none )



---- VIEW ----


effectDropdownOption : Effect -> Html Msg
effectDropdownOption effect =
    option [ value effect.key ] [ text effect.name ]


effectDropdown : List Effect -> Html Msg
effectDropdown effects =
    div []
        [ h2 [] [ text "Effect" ]
        , select [ onInput EffectChange ] (List.map effectDropdownOption effects)
        ]


slider : String -> HclParam -> ColorValue -> Html Msg
slider title hclparam colorvalue =
    div [ class "slider" ]
        [ h2 [] [ text title ]
        , h3 [] [ text (colorFractionString colorvalue) ]
        , input
            [ type_ "range"
            , Html.Attributes.min "0"
            , Html.Attributes.max "65535"
            , value (toString colorvalue)
            , onInput (HclChange hclparam)
            ]
            []
        ]


view : Model -> Html Msg
view model =
    div []
        [ h1 [] [ text "Any Colour You Like" ]
        , effectDropdown effects
        , div [ class "sliders" ]
            [ slider "Hue" Hue model.hue
            , slider "Chroma" Chroma model.chroma
            , slider "Luminance" Luminance model.luminance
            ]
        ]



---- PROGRAM ----


subscriptions model =
    WebSocket.listen "ws://nova:8011/" ServerState


main : Program Never Model Msg
main =
    Html.program
        { view = view
        , init = init
        , update = update
        , subscriptions = subscriptions
        }
