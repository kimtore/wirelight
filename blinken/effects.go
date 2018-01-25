package main

/*
func emergency(canvas *ledclient.Canvas) {
	blue := colorful.LinearRgb(0, 0, 1.0).Clamped()
	black := colorful.Hcl(0, 0, 0).Clamped()
	_ = black
	_, height := canvas.Size()
	half := height / 2
	offset := 0

	for {
		effect.Fill(canvas, black)
		offset = (offset + half) % b.Max.Y
		for blinks := 0; blinks < 2; blinks++ {
			for x := b.Min.X; x < b.Max.X; x++ {
				for y := offset; y < offset+half; y++ {
					canvas.Set(x, y, blue)
				}
			}
			time.Sleep(time.Millisecond * 20)
			for x := b.Min.X; x < b.Max.X; x++ {
				for y := offset; y < offset+half; y++ {
					canvas.Set(x, y, black)
				}
			}
			time.Sleep(time.Millisecond * 20)
			for x := b.Min.X; x < b.Max.X; x++ {
				for y := offset; y < offset+half; y++ {
					canvas.Set(x, y, blue)
				}
			}
		}
		time.Sleep(time.Millisecond * 20)
	}
}

func northernLights(canvas *ledclient.Canvas) {
	width, height := canvas.Size()
	old := make([]colorful.Color, b.Max.X*b.Max.Y)
	for {
		for angle := 0.0; angle < 360.0; angle++ {
			for x := 0; x < width; x++ {
				for y := 0; y < height; y++ {
					i := (y * b.Max.X) + x
					col := colorful.Hsl(angle+rand.Float64()*50.0, 1, rand.Float64()*0.1)
					step := col.BlendHcl(old[i], 0.92).Clamped()
					canvas.Set(x, y, step)
					old[i] = step
				}
			}
			time.Sleep(time.Millisecond * 50)
		}
	}
}

func northernLightsStable(canvas *ledclient.Canvas) {
	angle := 80.0
	angle = 180.0
	def := colorful.Hcl(angle, 1.0, 0.05)
	effect.Fill(canvas, def)
	for {
		effect.FillFunc(canvas, func(x, y int, c colorful.Color) colorful.Color {
			if rand.Intn(100) != 0 {
				return def.BlendRgb(c, 0.98)
			}
			a := 180.0 * (1.0 / float64(rand.Intn(500)+1))
			return colorful.Hcl(angle+a, 1, rand.Float64()*0.1)
		})
		time.Sleep(time.Millisecond * 10)
	}
}

func black(canvas *ledclient.Canvas) {
	effect.Fill(canvas, colorful.Hsv(0, 0, 0))
}

func white(canvas *ledclient.Canvas) {
	effect.Fill(canvas, colorful.Hsv(0, 0, 1.0))
}

func snake(canvas *ledclient.Canvas) {
	col := colorful.Hcl(0, 1, 0.1)
	black(canvas)
	b := canvas.Bounds()
	for {
		for y := b.Min.Y; y < b.Max.Y; y++ {
			for x := b.Min.X; x < b.Max.X; x++ {
				canvas.Set(x, y, col)
				time.Sleep(time.Millisecond * 40)
			}
		}
	}
}

func blinkWhite(canvas *ledclient.Canvas) {
	for {
		effect.Fill(canvas, colorful.Hcl(0, 0, 1.0))
		time.Sleep(time.Millisecond * 1000)
		effect.Fill(canvas, colorful.Hcl(0, 0, 0))
		time.Sleep(time.Millisecond * 1000)
	}
}

func split(canvas *ledclient.Canvas) {
	l := 0.15
	left := colorful.Hcl(30.0, 1.0, l).Clamped()
	right := colorful.Hcl(180.0, 1.0, l).Clamped()
	b := canvas.Bounds()
	half := b.Max.Y / 2

	for {
		for x := b.Min.X; x < b.Max.X; x++ {
			for y := b.Min.Y; y < half; y++ {
				canvas.Set(x, y, left)
			}
			for y := half; y < b.Max.Y; y++ {
				canvas.Set(x, y, right)
			}
		}
		time.Sleep(time.Millisecond * 1000)
	}
}

func fullBlue(canvas *ledclient.Canvas) {
	for {
		col := colorful.Hcl(80, 1.0, 1.0)
		effect.Fill(canvas, col)
		time.Sleep(time.Microsecond * 1000)
	}
}

func superGradients(canvas *ledclient.Canvas) {
	for {
		hue := rand.Float64() * 360.0
		for deg := 0.0; deg <= 180.0; deg += 1 {
			l := math.Sin(lib.Rad(deg))
			col := colorful.Hsv(hue, 1.0, l*0.5).Clamped()
			effect.Fill(canvas, col)
			time.Sleep(time.Microsecond * 1500)
		}
		time.Sleep(time.Millisecond * 185)
	}
}

// directionTest draws up a gradient on each strip.
func directionTest(canvas *ledclient.Canvas) {
	c := 1.0
	l := 0.05

	src := colorful.Hcl(0.0, c, l)
	dst := colorful.Hcl(160.0, c, l)
	b := canvas.Bounds()
	count := b.Max.X - b.Min.X
	step := float64(1.0) / float64(count)

	for {
		for y := b.Min.Y; y < b.Max.Y; y++ {
			n := 0.0
			for x := b.Min.X; x < b.Max.X; x++ {
				n += step
				col := src.BlendHcl(dst, n).Clamped()
				canvas.Set(x, y, col)
			}
		}
		time.Sleep(time.Millisecond * 1000)
	}
}

func gradients(canvas *ledclient.Canvas) {
	var h, c, l float64
	h = 0.0
	c = 0.8
	l = 0.5
	_, _ = c, l
	src := colorful.Hsv(h, 1, 1)
	dst := colorful.Hsv(h, 1, 1)

	for {
		src = dst
		h += 30
		if h >= 360 {
			h = 0
		}
		dst = colorful.Hsv(h, 1, 1)
		fmt.Printf("hue=%.2f, blend %#v %#v\n", h, src, dst)

		// interpolate between the two colors.
		for n := 0.0; n < 1.0; n += 0.01 {
			col := src.BlendHcl(dst, n).Clamped()
			effect.Fill(canvas, col)
			time.Sleep(time.Millisecond * 20)
		}
	}
}

func staccatoWheel(canvas *ledclient.Canvas) {
	var h float64
	for {
		h += 31
		if h > 360 {
			h -= 360
		}
		col := colorful.Hsv(h, 1, 0.25).Clamped()
		effect.Fill(canvas, col)
		time.Sleep(time.Millisecond * 250)
	}
}

func wheelHCL(canvas *ledclient.Canvas) {
	var h float64
	for {
		h += 1
		if h > 360 {
			h = 0
		}
		col := colorful.Hcl(h, 0.2, 0).Clamped()
		effect.Fill(canvas, col)
		time.Sleep(time.Millisecond * 100)
	}
}

func wheelHSV(canvas *ledclient.Canvas) {
	var h float64
	for {
		h += 1
		if h > 360 {
			h = 0
		}
		col := colorful.Hsv(h, 1, 1)
		effect.Fill(canvas, col)
		time.Sleep(time.Millisecond * 50)
	}
}
*/
