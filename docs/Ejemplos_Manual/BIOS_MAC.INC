;*************************************************************
; BIOS_MAC.INC
;
; Este archivo contiene macros permiten desplegar un carácter
; a pantalla y leer un carácter del teclado. Las macros
; utilizan llamadas a las rutinas del BIOS
;*************************************************************

;*************************************************************
; APUTCH
;
; Esta macro despliega un carácter en la pantalla
;
; Parámetro:
;
;        AL = carácter
;

; Regresa:
;
;       Nada
;*************************************************************
macro   aputch
        mov     ah, 0Eh                 ;; Despliega un carácter
        mov     bh, 0                   ;; en la página 0
        int     10h                     ;; Interrupción de video
endm    aputch

;*************************************************************
; AGETCH
;
; Esta macro lee un carácter del teclado
;
; Parámetro:
;
;        Nada
;
;
;
; Regresa:
;
;        AH = Código de la tecla
;        AL = Código ASCII del carácter
;*************************************************************
macro   agetch
        mov     ah, 10h                 ;; Leer un carácter
        int     16h                     ;; Interrupción del teclado
endm    agetch
