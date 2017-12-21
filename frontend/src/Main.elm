module Main exposing (..)

import Html exposing (Html, text, div, h1, h2, input)
import Html.Attributes exposing (src, type_, class, min, max, value)
import Html.Events exposing (onInput)


---- MODEL ----


type alias Model =
    { hue : ColorParam
    , chroma : ColorParam
    , luminance : ColorParam
    }


type alias ColorParam =
    Int


init : ( Model, Cmd Msg )
init =
    ( { hue = 0
      , chroma = 0
      , luminance = 0
      }
    , Cmd.none
    )



---- UPDATE ----


type HclParam
    = Hue
    | Chroma
    | Luminance


type Msg
    = NoOp
    | HclChange HclParam String


colorFraction : ColorParam -> Float
colorFraction c =
    toFloat c / 65535


colorFractionString : ColorParam -> String
colorFractionString c =
    String.left 6 (toString (colorFraction c))


zint : String -> ColorParam
zint s =
    Result.withDefault 0 (String.toInt s)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        HclChange Hue s ->
            ( { model | hue = zint s }, Cmd.none )

        HclChange Chroma s ->
            ( { model | chroma = zint s }, Cmd.none )

        HclChange Luminance s ->
            ( { model | luminance = zint s }, Cmd.none )

        NoOp ->
            ( model, Cmd.none )



---- VIEW ----


slider : String -> HclParam -> ColorParam -> Html Msg
slider title hclparam colorvalue =
    div [ class "slider" ]
        [ h2 [] [ text (title ++ ": " ++ colorFractionString colorvalue) ]
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
        , slider "Hue" Hue model.hue
        , slider "Chroma" Chroma model.chroma
        , slider "Luminance" Luminance model.luminance
        ]



---- PROGRAM ----


main : Program Never Model Msg
main =
    Html.program
        { view = view
        , init = init
        , update = update
        , subscriptions = always Sub.none
        }
